#!/bin/sh
set -eu

find src -type f -name '*.rs' -exec sh -c '
  target_limit=300
  hard_limit=400
  failed=0

  for file do
    lines=$(wc -l < "$file" | tr -d " ")
    if [ "$lines" -gt "$hard_limit" ]; then
      echo "error: $file has $lines lines; hard limit is $hard_limit" >&2
      failed=1
    elif [ "$lines" -gt "$target_limit" ]; then
      echo "warning: $file has $lines lines; target is $target_limit" >&2
    fi
  done

  exit "$failed"
' sh {} +
