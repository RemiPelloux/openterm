#!/bin/sh
set -eu

destination=${1:-target/debug}
binary="$destination/openterm"

if [ ! -x "$binary" ]; then
  echo "error: build OpenTerm before linking commands" >&2
  exit 1
fi

"$binary" --print-command-names | while IFS= read -r command; do
  ln -sfn openterm "$destination/$command"
done
