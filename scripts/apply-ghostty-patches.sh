#!/bin/sh
# Apply OpenTerm branding and behavior patches to a Ghostty source tree.
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
tree=${1:?usage: apply-ghostty-patches.sh GHOSTTY_SOURCE_DIR}

if [ ! -f "$tree/build.zig" ]; then
  echo "error: $tree is not a Ghostty source root" >&2
  exit 1
fi

cd "$tree"
for patch in "$root"/native/macos/patches/*.patch; do
  [ -f "$patch" ] || continue
  patch -p1 -N -r - < "$patch"
done

appdelegate="$tree/macos/Sources/App/macOS/AppDelegate.swift"
if [ -f "$appdelegate" ] && grep -q '#if DEBUG' "$appdelegate"; then
  perl -i -0pe 's/#if DEBUG\n(\s*)ghostty = Ghostty\.App\(configPath: ProcessInfo\.processInfo\.environment\["GHOSTTY_CONFIG_PATH"\]\)\n#else\n\s*ghostty = Ghostty\.App\(\)\n#endif/$1ghostty = Ghostty.App(configPath: ProcessInfo.processInfo.environment["GHOSTTY_CONFIG_PATH"])/s' \
    "$appdelegate"
elif [ -f "$appdelegate" ] && ! grep -q 'GHOSTTY_CONFIG_PATH' "$appdelegate"; then
  echo "error: AppDelegate.swift no longer matches OpenTerm config-path patch" >&2
  exit 1
fi

swift_root="$tree/macos/Sources"
if [ -d "$swift_root" ]; then
  find "$swift_root" -name '*.swift' -print0 | while IFS= read -r -d '' file; do
    sed -i '' \
      -e 's/"Quit Ghostty?"/"Quit OpenTerm?"/g' \
      -e 's/"Close Ghostty"/"Close OpenTerm"/g' \
      -e 's/"Allow Ghostty to execute/"Allow OpenTerm to execute/g' \
      -e 's/"Allow Shortcuts to interact with Ghostty?"/"Allow Shortcuts to interact with OpenTerm?"/g' \
      -e 's/"The Ghostty app isn'\''t properly initialized."/"The OpenTerm app isn'\''t properly initialized."/g' \
      -e 's/"Ghostty doesn'\''t allow Shortcuts."/"OpenTerm doesn'\''t allow Shortcuts."/g' \
      -e 's/restart Ghostty./restart OpenTerm./g' \
      -e 's/"Ghostty failed to initialize! If you'\''re executing Ghostty from the command line"/"OpenTerm failed to initialize! If you'\''re executing OpenTerm from the command line"/g' \
      -e 's/"Ghostty app delegate is unavailable."/"OpenTerm app delegate is unavailable."/g' \
      -e 's/"Ghostty Application Icon"/"OpenTerm Application Icon"/g' \
      -e 's/"Update Ghostty and Restart"/"Updates managed by OpenTerm"/g' \
      -e 's/"👻 Ghostty"/"◆ OpenTerm"/g' \
      -e 's/Ghostty could not be set as the default terminal application./OpenTerm could not be set as the default terminal application./g' \
      "$file"
  done
fi

about="$tree/macos/Sources/Features/About/AboutView.swift"
if [ -f "$about" ]; then
  sed -i '' \
    -e 's|https://github.com/ghostty-org/ghostty|https://github.com/openpro/openterm|g' \
    -e 's|https://ghostty.org/docs|https://github.com/openpro/openterm|g' \
    -e 's/Text("Ghostty")/Text("OpenTerm")/g' \
    -e 's/Fast, native, feature-rich terminal \\nemulator pushing modern features./Agent-first terminal powered by OpenTerm tools and RTK./g' \
    "$about"
fi

menu="$tree/macos/Sources/App/macOS/MainMenu.xib"
if [ -f "$menu" ]; then
  sed -i '' \
    -e 's/title="Ghostty"/title="OpenTerm"/g' \
    -e 's/About Ghostty/About OpenTerm/g' \
    -e 's/Make Ghostty the Default Terminal/Make OpenTerm the Default Terminal/g' \
    -e 's/Hide Ghostty/Hide OpenTerm/g' \
    -e 's/Quit Ghostty/Quit OpenTerm/g' \
    -e 's/Ghostty Help/OpenTerm Help/g' \
    -e 's/title="Check for Updates\.\.\."/title="Updates managed by OpenTerm" enabled="NO"/g' \
    "$menu"
fi

pbx="$tree/macos/Ghostty.xcodeproj/project.pbxproj"
if [ -f "$pbx" ]; then
  sed -i '' \
    -e 's/running within Ghostty/running within OpenTerm/g' \
    -e 's/INFOPLIST_KEY_CFBundleDisplayName = Ghostty;/INFOPLIST_KEY_CFBundleDisplayName = OpenTerm;/g' \
    -e 's/INFOPLIST_KEY_CFBundleName = Ghostty;/INFOPLIST_KEY_CFBundleName = OpenTerm;/g' \
    "$pbx"
fi

echo "OpenTerm patches applied under $tree"
