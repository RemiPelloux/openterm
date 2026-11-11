#!/bin/sh
# Installs a release build of OpenTerm: copies the multicall binary and links
# every ai-* command next to it. Usage: ./install.sh [PREFIX] (default ~/.local)
set -eu

prefix=${1:-"$HOME/.local"}
bin="$prefix/bin"
here=$(cd "$(dirname "$0")" && pwd)

mkdir -p "$bin"
install -m 0755 "$here/openterm" "$bin/openterm"
"$bin/openterm" --print-command-names | while IFS= read -r command; do
  ln -sfn openterm "$bin/$command"
done

count=$("$bin/openterm" --print-command-names | wc -l | tr -d ' ')
echo "Installed openterm and $count ai-* commands into $bin"

case ":$PATH:" in
  *":$bin:"*) ;;
  *) echo "Add $bin to your PATH, e.g.: echo 'export PATH=\"$bin:\$PATH\"' >> ~/.zshrc" ;;
esac
command -v rtk >/dev/null 2>&1 || echo "Note: install RTK 0.43+ for the token-aware wrappers (ai-test, ai-grep, ...)."
echo "Next: ai-connect   (or ai-doctor to check your setup)"
