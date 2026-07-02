.PHONY: app build check clone-ghostty commands format release terminal test verify

app: release
	./scripts/package-openterm-macos.sh

clone-ghostty:
	./scripts/clone-ghostty.sh

terminal:
	./scripts/build-terminal-macos.sh

build:
	./scripts/cargo.sh build
	./scripts/link-commands.sh target/debug

commands:
	./scripts/link-commands.sh target/debug

release:
	./scripts/cargo.sh build --release
	./scripts/link-commands.sh target/release

check:
	./scripts/cargo.sh check --all-targets

format:
	./scripts/cargo.sh fmt --all

test:
	./scripts/cargo.sh test --all-targets

verify:
	./scripts/verify.sh
