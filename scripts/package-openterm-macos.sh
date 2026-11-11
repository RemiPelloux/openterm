#!/bin/sh
set -eu

. "$(dirname "$0")/native-common.sh"

app="$root/dist/OpenTerm.app"
plist="$app/Contents/Info.plist"
resources="$app/Contents/Resources"

resolve_terminal_base() {
  if [ -n "${GHOSTTY_APP:-}" ] && [ -d "$GHOSTTY_APP" ]; then
    echo "$GHOSTTY_APP"
    return
  fi
  if [ "${OPENTERM_USE_DMG:-0}" = "1" ]; then
    dmg="$downloads/Ghostty-$GHOSTTY_VERSION.dmg"
    mount="$cache/mount"
    base="$cache/base/Ghostty.app"
    download "$GHOSTTY_DMG_URL" "$dmg"
    verify "$GHOSTTY_DMG_SHA256" "$dmg"
    if [ ! -d "$base" ]; then
      mkdir -p "$(dirname "$base")"
      rm -rf "$mount"
      mkdir -p "$mount"
      hdiutil attach "$dmg" -mountpoint "$mount" -nobrowse -readonly >/dev/null
      trap 'hdiutil detach "$mount" >/dev/null 2>&1 || true' EXIT HUP INT TERM
      ditto "$mount/Ghostty.app" "$base"
      hdiutil detach "$mount" >/dev/null
      trap - EXIT HUP INT TERM
    fi
    echo "$base"
    return
  fi
  prepare_native_sources >&2
  "$root/scripts/build-terminal-macos.sh"
}

base=$(resolve_terminal_base)
codesign --verify --deep --strict "$base" 2>/dev/null || true

rm -rf "$app"
mkdir -p "$root/dist"
ditto "$base" "$app"

if [ -f "$app/Contents/MacOS/ghostty" ]; then
  mv "$app/Contents/MacOS/ghostty" "$app/Contents/MacOS/openterm-terminal"
elif [ ! -f "$app/Contents/MacOS/openterm-terminal" ]; then
  echo "error: terminal payload executable was not found" >&2
  exit 1
fi
xcrun clang -fobjc-arc -framework Foundation -mmacosx-version-min=13.0 \
  "$root/native/macos/launcher.m" -o "$app/Contents/MacOS/openterm"

bundle="$resources/openterm"
bin="$bundle/bin"
rm -rf "$bundle"
mkdir -p "$bin" "$bundle/licenses" "$bundle/shell" "$cache/swift-modules"
cp "$root/target/release/openterm" "$bin/openterm"
"$bin/openterm" --print-command-names | while IFS= read -r command; do
  ln -s openterm "$bin/$command"
done
xcrun swiftc -O -parse-as-library -module-cache-path "$cache/swift-modules" \
  -framework SwiftUI -framework AppKit \
  "$root/native/macos/OpenTermUI.swift" \
  "$root/native/macos/AIPane.swift" \
  "$root/native/macos/ProviderDetail.swift" \
  "$root/native/macos/ProviderStore.swift" \
  "$root/native/macos/OpenTermTools.swift" \
  -o "$bin/openterm-ui"
ln -s openterm-ui "$bin/OpenTermUI"
xcrun swiftc -O -parse-as-library -module-cache-path "$cache/swift-modules" \
  -framework SwiftUI -framework AppKit -framework CoreGraphics \
  "$root/native/macos/AgentPanel.swift" \
  "$root/native/macos/PanelStreams.swift" \
  "$root/native/macos/OpenTermTools.swift" \
  -o "$bin/openterm-panel"
xcrun swiftc -O -module-cache-path "$cache/swift-modules" -framework AppKit \
  "$root/native/macos/Splash.swift" -o "$bundle/openterm-splash"

# Prefer the pinned RTK on PATH; otherwise reuse the RTK an earlier OpenTerm
# package already verified and bundled (it is re-signed, so its hash differs).
rtk=$(command -v rtk || true)
base_rtk="$base/Contents/Resources/openterm/bin/rtk"
if [ -n "$rtk" ] && [ "$("$rtk" --version)" = "rtk $RTK_VERSION" ]; then
  verify "$RTK_SHA256" "$rtk"
elif [ -x "$base_rtk" ] && [ "$("$base_rtk" --version)" = "rtk $RTK_VERSION" ]; then
  rtk=$base_rtk
else
  echo "error: RTK $RTK_VERSION is required to package OpenTerm" >&2
  exit 1
fi
cp "$rtk" "$bin/rtk"

cp "$root/native/macos/config.ghostty" "$bundle/default-config.ghostty"
cp -R "$root/native/macos/shell/." "$bundle/shell/"
cp "$root/shell/openterm.zsh" "$bundle/shell/openterm.zsh"
cp "$root/shell/openterm.bash" "$bundle/shell/openterm.bash"
chmod +x "$bundle/shell/openterm-shell"
cp "$root/LICENSE" "$bundle/licenses/OPENTERM-LICENSE"
cp "$root/native/licenses/GHOSTTY-LICENSE" "$bundle/licenses/"
cp "$root/THIRD_PARTY_NOTICES.md" "$bundle/licenses/"

icon_png="$cache/OpenTerm-1024.png"
iconset="$cache/OpenTerm.iconset"
rm -rf "$iconset"
mkdir -p "$iconset"
swift "$root/native/macos/generate-icon.swift" "$icon_png"
make_icon() {
  pixels=$1
  name=$2
  sips -z "$pixels" "$pixels" "$icon_png" --out "$iconset/$name" >/dev/null
}
make_icon 16 icon_16x16.png
make_icon 32 icon_16x16@2x.png
make_icon 32 icon_32x32.png
make_icon 64 icon_32x32@2x.png
make_icon 128 icon_128x128.png
make_icon 256 icon_128x128@2x.png
make_icon 256 icon_256x256.png
make_icon 512 icon_256x256@2x.png
make_icon 512 icon_512x512.png
cp "$icon_png" "$iconset/icon_512x512@2x.png"
iconutil -c icns "$iconset" -o "$resources/OpenTerm.icns"

/usr/libexec/PlistBuddy -c 'Set :CFBundleExecutable openterm' "$plist"
/usr/libexec/PlistBuddy -c 'Set :CFBundleName OpenTerm' "$plist"
/usr/libexec/PlistBuddy -c 'Set :CFBundleDisplayName OpenTerm' "$plist"
/usr/libexec/PlistBuddy -c 'Set :CFBundleIdentifier com.openpro.openterm' "$plist"
/usr/libexec/PlistBuddy -c 'Set :CFBundleIconFile OpenTerm' "$plist"
/usr/libexec/PlistBuddy -c 'Delete :CFBundleIconName' "$plist" 2>/dev/null || true
/usr/libexec/PlistBuddy -c 'Delete :NSDockTilePlugIn' "$plist" 2>/dev/null || true
/usr/libexec/PlistBuddy -c 'Delete :SUPublicEDKey' "$plist" 2>/dev/null || true
/usr/libexec/PlistBuddy -c 'Add :SUEnableAutomaticChecks bool false' "$plist" 2>/dev/null || \
  /usr/libexec/PlistBuddy -c 'Set :SUEnableAutomaticChecks false' "$plist"
/usr/libexec/PlistBuddy -c 'Add :SUAutomaticallyUpdate bool false' "$plist" 2>/dev/null || \
  /usr/libexec/PlistBuddy -c 'Set :SUAutomaticallyUpdate false' "$plist"
/usr/libexec/PlistBuddy -c "Add :OpenTermTerminalBaseVersion string $GHOSTTY_VERSION" "$plist" 2>/dev/null || \
  /usr/libexec/PlistBuddy -c "Set :OpenTermTerminalBaseVersion $GHOSTTY_VERSION" "$plist"
/usr/libexec/PlistBuddy -c 'Add :OpenTermEngineSource string source' "$plist" 2>/dev/null || \
  /usr/libexec/PlistBuddy -c 'Set :OpenTermEngineSource source' "$plist"

for key in \
  NSAppleEventsUsageDescription \
  NSBluetoothAlwaysUsageDescription \
  NSCalendarsUsageDescription \
  NSCameraUsageDescription \
  NSContactsUsageDescription \
  NSLocalNetworkUsageDescription \
  NSLocationUsageDescription \
  NSMicrophoneUsageDescription \
  NSMotionUsageDescription \
  NSPhotoLibraryUsageDescription \
  NSRemindersUsageDescription \
  NSSpeechRecognitionUsageDescription \
  NSSystemAdministrationUsageDescription; do
  value=$(/usr/libexec/PlistBuddy -c "Print :$key" "$plist" 2>/dev/null || true)
  case "$value" in
    *Ghostty*)
      fixed=$(printf '%s' "$value" | sed 's/Ghostty/OpenTerm/g')
      /usr/libexec/PlistBuddy -c "Set :$key $fixed" "$plist"
      ;;
  esac
done

# Apple deprecates --deep for signing. Seal nested code from the inside out so
# every nested bundle carries its own signature before the outer one is sealed.
sign() {
  codesign --force --sign - --timestamp=none "$1"
}

find "$app/Contents" -depth \
  \( -name '*.framework' -o -name '*.app' -o -name '*.xpc' -o -name '*.bundle' \) \
  -print | while IFS= read -r nested; do
  sign "$nested"
done

for tool in "$bin"/* "$bundle/openterm-splash" "$app/Contents/MacOS"/*; do
  if [ -f "$tool" ] && [ ! -L "$tool" ]; then
    sign "$tool"
  fi
done

sign "$app"
codesign --verify --deep --strict --verbose=2 "$app"
"$app/Contents/MacOS/openterm-terminal" +validate-config \
  --config-file="$bundle/default-config.ghostty"

release_dmg="$root/dist/OpenTerm-0.1.0-alpha-macos-arm64.dmg"
rm -f "$release_dmg"
hdiutil create -quiet -volname OpenTerm -srcfolder "$app" -ov -format UDZO "$release_dmg"
echo "$app"
echo "$release_dmg"
