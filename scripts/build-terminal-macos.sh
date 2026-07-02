#!/bin/sh
set -eu

. "$(dirname "$0")/native-common.sh"
prepare_native_sources

gettext_prefix=$(brew --prefix gettext)
PATH="$zig_source:$gettext_prefix/bin:$PATH"
export PATH

echo "Building Ghostty $GHOSTTY_VERSION terminal base..."
cd "$ghostty_source"
patch -p1 -N -r - < "$root/native/macos/patches/ghostty-config-path.patch" || true
zig build -Doptimize=ReleaseFast -Dtarget="$MACOS_TARGET"

app="$ghostty_source/zig-out/Ghostty.app"
if [ ! -d "$app" ]; then
  echo "error: Ghostty build did not produce $app" >&2
  exit 1
fi
echo "$app"
