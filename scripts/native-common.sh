#!/bin/sh

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
. "$root/native/ghostty.lock"

cache="$root/.cache/native"
downloads="$cache/downloads"
ghostty_source="$cache/ghostty-$GHOSTTY_VERSION"
zig_source="$cache/zig-aarch64-macos-$ZIG_VERSION"

download() {
  url=$1
  destination=$2
  if [ ! -f "$destination" ]; then
    curl --fail --location --progress-bar "$url" --output "$destination"
  fi
}

verify() {
  expected=$1
  file=$2
  actual=$(shasum -a 256 "$file" | awk '{print $1}')
  if [ "$actual" != "$expected" ]; then
    echo "error: checksum mismatch for $file" >&2
    exit 1
  fi
}

prepare_native_sources() {
  mkdir -p "$downloads"
  zig_archive="$downloads/zig-$ZIG_VERSION.tar.xz"
  ghostty_archive="$downloads/ghostty-$GHOSTTY_VERSION.tar.gz"
  download "$ZIG_URL" "$zig_archive"
  download "$GHOSTTY_URL" "$ghostty_archive"
  verify "$ZIG_SHA256" "$zig_archive"
  verify "$GHOSTTY_SHA256" "$ghostty_archive"
  if [ ! -x "$zig_source/zig" ]; then
    tar -xJf "$zig_archive" -C "$cache"
  fi
  if [ ! -f "$ghostty_source/build.zig" ]; then
    tar -xzf "$ghostty_archive" -C "$cache"
  fi
}
