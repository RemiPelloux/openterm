#!/bin/sh
# Clone ghostty-org/ghostty into native/ghostty and apply OpenTerm branding in-tree.
set -eu

. "$(dirname "$0")/native-common.sh"

ghostty_dir="$root/native/ghostty"
patch_rev=${OPENTERM_PATCH_REVISION:-0}
brand_stamp="$ghostty_dir/.openterm-branding-$GHOSTTY_REVISION-$patch_rev"

if [ ! -d "$ghostty_dir/.git" ]; then
  echo "Cloning Ghostty $GHOSTTY_VERSION from $GHOSTTY_GIT_URL ..."
  rm -rf "$ghostty_dir"
  mkdir -p "$(dirname "$ghostty_dir")"
  git clone --filter=blob:none "$GHOSTTY_GIT_URL" "$ghostty_dir"
  cd "$ghostty_dir"
  git fetch --tags origin 2>/dev/null || true
  git checkout --force "$GHOSTTY_REVISION"
elif [ "${GHOSTTY_FORCE_SYNC:-0}" = "1" ]; then
  cd "$ghostty_dir"
  git fetch --tags origin
  git checkout --force "$GHOSTTY_REVISION"
  rm -f "$ghostty_dir"/.openterm-branding-*
else
  cd "$ghostty_dir"
fi

actual=$(git rev-parse HEAD)
if [ "$actual" != "$GHOSTTY_REVISION" ]; then
  echo "warning: native/ghostty is at $actual (lock pins $GHOSTTY_REVISION)" >&2
  echo "Set GHOSTTY_FORCE_SYNC=1 to reset upstream and re-apply OpenTerm branding." >&2
fi

if [ ! -f "$brand_stamp" ]; then
  "$root/scripts/apply-ghostty-patches.sh" "$ghostty_dir"
  rm -f "$ghostty_dir"/.openterm-branding-*
  touch "$brand_stamp"
fi

echo "OpenTerm engine source ready at $ghostty_dir ($actual)"
