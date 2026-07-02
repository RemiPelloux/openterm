#!/bin/sh
set -eu

./scripts/check-size.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh check --all-targets
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/cargo.sh test --all-targets

