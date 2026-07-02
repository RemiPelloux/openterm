#!/bin/sh
set -eu

if command -v cargo >/dev/null 2>&1; then
  exec cargo "$@"
fi

if command -v rustup >/dev/null 2>&1; then
  toolchain_bin=$(dirname "$(rustup which cargo)")
  PATH="$toolchain_bin:$PATH"
  export PATH
  exec "$toolchain_bin/cargo" "$@"
fi

echo "error: cargo is unavailable; install Rust with rustup" >&2
exit 127
