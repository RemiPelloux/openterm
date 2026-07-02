#!/bin/sh
set -eu

. "$(dirname "$0")/native-common.sh"
prepare_native_sources
prepare_ghostty_source

patch_rev=${OPENTERM_PATCH_REVISION:-0}
brand_stamp="$ghostty_dir/.openterm-branding-$GHOSTTY_REVISION-$patch_rev"
if [ ! -f "$brand_stamp" ]; then
  "$root/scripts/apply-ghostty-patches.sh" "$ghostty_dir"
  rm -f "$ghostty_dir"/.openterm-branding-*
  touch "$brand_stamp"
fi

gettext_prefix=$(brew --prefix gettext 2>/dev/null || true)
sdk_root=$(xcrun --sdk macosx --show-sdk-path)
zig_bin="$zig_source/zig"
if [ ! -x "$zig_bin" ]; then
  echo "error: Zig $ZIG_VERSION is missing under $zig_source" >&2
  exit 1
fi

build_target="$MACOS_TARGET"
if [ "$(uname -s)" = "Darwin" ] && [ "$(uname -m)" = "arm64" ]; then
  build_target="native"
fi

build_env() {
  env -i \
    HOME="$HOME" \
    USER="${USER:-$(id -un)}" \
    TMPDIR="${TMPDIR:-/tmp}" \
    SDKROOT="$sdk_root" \
    MACOSX_DEPLOYMENT_TARGET=13.0 \
    PATH="/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/bin${gettext_prefix:+:$gettext_prefix/bin}:${zig_source}" \
    "$@"
}

echo "Building OpenTerm terminal engine from native/ghostty ($GHOSTTY_REVISION, target=$build_target)..."

cd "$ghostty_dir"

# Ghostty CI builds GhosttyKit with Zig, then the macOS app with xcodebuild.
build_env "$zig_bin" build \
  -Doptimize=ReleaseFast \
  -Dtarget="$build_target" \
  -Demit-macos-app=false

build_env xcodebuild \
  -project macos/Ghostty.xcodeproj \
  -target Ghostty \
  -configuration ReleaseLocal \
  SYMROOT="$ghostty_dir/macos/build"

app="$ghostty_dir/macos/build/ReleaseLocal/Ghostty.app"
if [ ! -d "$app" ]; then
  echo "error: xcodebuild did not produce $app" >&2
  exit 1
fi

cache_app="$cache/base/Ghostty.app"
rm -rf "$cache_app"
mkdir -p "$(dirname "$cache_app")"
ditto "$app" "$cache_app"
echo "$cache_app"
