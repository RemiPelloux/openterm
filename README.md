# OpenTerm

OpenTerm is an AI-native terminal with persistent sessions, token-aware command execution, native RTK-derived compression, and capability-safe agents and plugins. Every OpenTerm-native tool command uses the `ai-` prefix, leaving system commands untouched.

The repository now produces a native macOS terminal application with a
Metal/CoreText terminal engine, tabs, splits, shell integration, encrypted
evidence references, bundled RTK, provider adapters, and the complete current
`ai-*` suite. The dedicated OpenTerm session daemon and AI sidecar remain under
development.

## Why Rust

Rust is the primary language because OpenTerm needs reliable PTY/process control, a persistent daemon, low-latency terminal integration, safe concurrency, and direct reuse of RTK's Rust filtering work. The project starts as one binary with only the HTTP/JSON dependencies required by `ai-ask`; it will split into crates only when real ownership boundaries emerge.

## Requirements

- Rust stable with `rustfmt` and Clippy
- Git
- RTK 0.43 for CLI development; the macOS app bundles it
- Codex CLI for the default `ai-ask` provider
- macOS or Linux for the initial development target

Install Rust through [rustup](https://rustup.rs/), then verify:

```sh
rustup component add rustfmt clippy
make verify
```

If `cargo` is not directly available but `rustup` is, the repository's `scripts/cargo.sh` wrapper uses `rustup run stable cargo` automatically.

## Development

```sh
make build
./target/debug/openterm
./target/debug/ai-version
./target/debug/ai-doctor
./target/debug/ai-ask "what command shows available disk space?"
./target/debug/ai-run df -h
./target/debug/ai-grep "TODO" src
./target/debug/ai-tree -L 2
./target/debug/ai-status
make verify
```

Create an optimized build and all `ai-*` command links with:

```sh
make release
./target/release/openterm
./target/release/openterm mascot
./target/release/OpenTermUI
```

Build and launch the real macOS terminal app:

```sh
make app
open dist/OpenTerm.app
```

The local alpha DMG is written to
`dist/OpenTerm-0.1.0-alpha-macos-arm64.dmg`. It is ad-hoc signed for local
testing, not Developer ID signed or notarized for public distribution.

RTK-backed commands now print a stable evidence reference on stderr:

```text
openterm: result res_ab12... (expand with: ai-expand res_ab12...)
```

Inspect and retrieve it without rerunning the command:

```sh
ai-result res_ab12... --json
ai-expand res_ab12...
ai-page res_ab12... --offset 0 --limit 4096
ai-pin res_ab12...
ai-forget res_ab12...
```

## Implemented commands

| Command | Backend | Purpose |
|---|---|---|
| `ai-ask` | LLM provider | Read-only terminal guidance |
| `ai-doctor` | OpenTerm | Environment and provider diagnostics |
| `ai-version` | OpenTerm | Version output |
| `ai-grep` | `rtk grep` | Grouped token-efficient search |
| `ai-find` | `rtk find` | Compact file discovery |
| `ai-tree` | `rtk tree` | Token-efficient directory tree |
| `ai-read` | `rtk read` | Bounded intelligent file reading |
| `ai-status` | `rtk git status` | Compact Git status |
| `ai-diff` | `rtk git diff` | Condensed Git diff |
| `ai-test` | `rtk test` | Failure-focused tests |
| `ai-log` | `rtk log` | Deduplicated logs |
| `ai-result` | OpenTerm | Inspect evidence metadata |
| `ai-expand` | OpenTerm | Retrieve exact stored evidence |
| `ai-page` | OpenTerm | Read bounded evidence ranges |
| `ai-export` | OpenTerm | Export evidence to a file |
| `ai-pin` | OpenTerm | Protect evidence from cleanup |
| `ai-forget` | OpenTerm | Move evidence to recoverable trash |
| `ai-run` | OpenTerm | Execute safely and preserve raw output |
| `ai-history` | OpenTerm | Browse scored local command history |
| `ai-search-history` | OpenTerm | Search argv, command, and workspace paths |
| `ai-clean-history` | OpenTerm | Explain cleanup candidates without deleting |
| `ai-usage` | OpenTerm | Show evidence and expansion statistics |
| `ai-context` | OpenTerm | Git branch, changes, divergence, conflicts, and stashes |
| `ai-ui` / `OpenTermUI` | Native macOS | Open live appearance and theme settings |

The current RTK bridge captures the exact RTK stdout and stderr, preserves its
exit code, encrypts both streams, and identifies them as
`rtk-filtered-output`. Native RTK extraction is required before OpenTerm can
capture the underlying tool's pre-filter stream.

## Local evidence store

Metadata uses SQLite WAL. Evidence blobs use XChaCha20-Poly1305 with a private
mode-0600 local key. Set `OPENTERM_DATA_DIR` to override the default data path.
The key moves to macOS Keychain and Linux Secret Service before public release.
History retention scoring is documented in `docs/synaptic-retention.md`.

The launcher respects `NO_COLOR` and `OPENTERM_REDUCED_MOTION`. Run
`openterm mascot` to preview the canonical 60 ms jump/wave animation.

Inside the native app, zsh and Bash retain the user’s existing configuration
and gain an OpenTerm layer: a compact welcome, dynamic window title, `ai-cd`,
and live Git branch/staged/modified/renamed/untracked/conflict/ahead/behind/stash
information. Run `OpenTermUI` or `ai-ui` to edit the theme, font, colors,
opacity, glass blur, padding, and titlebar with a live preview.

Source the appropriate integration to enable `ai-cd` in the parent shell:

```sh
source /path/to/OpenTerm/shell/openterm.zsh
# or
source /path/to/OpenTerm/shell/openterm.bash
```

## LLM providers

`ai-ask` is a read-only terminal adviser. Codex is the default provider and runs through `codex exec` with ephemeral, read-only settings. API providers require an explicit model and the corresponding credential:

```sh
ai-ask "what is using the most disk space here?"
ai-ask --provider openai --model <model-id> "explain df -h"
ai-ask --provider claude --model <model-id> "explain ps aux"
ai-ask --provider mistral --model <model-id> "explain du -sh"
```

| Provider | Credential |
|---|---|
| OpenAI Responses API | `OPENAI_API_KEY` |
| Claude Messages API | `ANTHROPIC_API_KEY` |
| Mistral Chat Completions API | `MISTRAL_API_KEY` |
| Codex CLI | Existing `codex login` session or Codex-supported API authentication |

API credentials are read from the process environment and never printed by `ai-doctor`. Behavioral provider/model defaults will live in OpenTerm configuration rather than new environment variables.

Enable the repository pre-commit hook once:

```sh
git config core.hooksPath .githooks
```

## Repository map

```text
src/                  OpenTerm CLI and encrypted evidence foundation
tests/                CLI behavior tests
docs/                 Architecture and provider contracts
design/mascot/        Canonical mascot art and animation data
shell/                zsh and Bash parent-shell integrations
native/               Pinned terminal inputs and macOS packaging sources
scripts/              Portable Cargo and verification wrappers
.cursor/rules/        Canonical Pelloux engineering discipline
roadmap.md             Product, architecture, and delivery roadmap
```

Current delivery status is tracked in `docs/implementation-status.md`.

## Engineering rules

The canonical local rules are in `.cursor/rules/pelloux-guidelines.mdc`. Production Rust files target 300 lines and fail above 400; functions target 25 lines; APIs accept at most four parameters. `make verify` enforces formatting, compilation, Clippy, tests, and file-size limits.

All new user-facing OpenTerm tool executables must begin with `ai-`. The `openterm` executable is reserved for launching and controlling the application.

## License

OpenTerm is licensed under Apache-2.0. Third-party and mascot provenance is
recorded in `THIRD_PARTY_NOTICES.md` and `design/mascot/README.md`.
