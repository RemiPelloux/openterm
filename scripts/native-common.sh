#!/bin/sh

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
. "$root/native/ghostty.lock"

cache="$root/.cache/native"
downloads="$cache/downloads"
ghostty_dir="$root/native/ghostty"
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

prepare_zig() {
  mkdir -p "$downloads"
  zig_archive="$downloads/zig-$ZIG_VERSION.tar.xz"
  download "$ZIG_URL" "$zig_archive"
  verify "$ZIG_SHA256" "$zig_archive"
  if [ ! -x "$zig_source/zig" ]; then
    tar -xJf "$zig_archive" -C "$cache"
  fi
}

prepare_native_sources() {
  prepare_zig
}

prepare_ghostty_source() {
  if [ ! -f "$ghostty_dir/build.zig" ]; then
    "$(dirname "$0")/clone-ghostty.sh"
  fi
}
