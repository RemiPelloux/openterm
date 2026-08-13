# OpenTerm

OpenTerm is an AI-native terminal with persistent sessions, token-aware command execution, native RTK-derived compression, and capability-safe agents and plugins. Every OpenTerm-native tool command uses the `ai-` prefix, leaving system commands untouched.

The repository produces a native macOS terminal application built on
[Ghostty](https://github.com/ghostty-org/ghostty) (branded and patched in-tree),
with shell integration, encrypted evidence references, bundled RTK, LLM provider
adapters, and the full `ai-*` command suite. The dedicated session daemon and AI
sidecar remain under development.

## Why Rust

Rust is the primary language because OpenTerm needs reliable PTY/process control, a persistent daemon, low-latency terminal integration, safe concurrency, and direct reuse of RTK's Rust filtering work. The project starts as one binary with only the HTTP/JSON dependencies required by `ai-ask`; it will split into crates only when real ownership boundaries emerge.

## Requirements

- Rust stable with `rustfmt` and Clippy
- Git
- RTK 0.43 for CLI development; the macOS app bundles it
- macOS for the native app (Linux supported for CLI development)
- **Optional (source engine build):** Xcode, Zig 0.15.2 (pinned in `native/ghostty.lock`), Homebrew `gettext`

Install Rust through [rustup](https://rustup.rs/), then verify:

```sh
rustup component add rustfmt clippy
make verify
```

If `cargo` is not directly available but `rustup` is, the repository's `scripts/cargo.sh` wrapper uses `rustup run stable cargo` automatically.

## Quick start (macOS app)

```sh
make app
open dist/OpenTerm.app
```

Install to `/Applications`:

```sh
ditto dist/OpenTerm.app /Applications/OpenTerm.app
```

The local alpha DMG is written to `dist/OpenTerm-0.1.0-alpha-macos-arm64.dmg`.
It is ad-hoc signed for local testing, not Developer ID signed or notarized.

### Connect an LLM provider

Inside OpenTerm, run the interactive setup wizard:

```sh
ai-connect
```

Use arrow keys to pick a provider (Codex, OpenAI, Claude, Mistral, Ollama,
LM Studio). Direct API providers then discover models and save the selected
default. Their keys are currently stored in `~/.config/openterm/providers.json`
(mode `600`), with environment variables taking precedence.

- **Codex** — delegates authentication to the installed Codex CLI and never
  copies its OAuth tokens; run `codex login` first
- **API providers** — prompt for an API key, then fetch live model lists.
  OpenAI-compatible gateways use `OPENAI_BASE_URL` or `base_url` in
  `providers.json` (origin only, without `/v1`)
- **Ollama / LM Studio** — no API key; start the local runtime, then pick a
  model it has already pulled. Override the host with `OLLAMA_HOST` or
  `LMSTUDIO_HOST` if it is not on the default port.

Check status without exposing secrets:

```sh
ai-connect --status
ai-doctor
```

## Development

```sh
make build
./target/debug/openterm
./target/debug/ai-version
./target/debug/ai-help
./target/debug/ai-connect --status
./target/debug/ai-ask "what command shows available disk space?"
./target/debug/ai-run df -h
./target/debug/ai-rg "TODO" src
make verify
```

Create an optimized build and all `ai-*` command links with:

```sh
make release
./target/release/openterm
./target/release/openterm mascot
./target/release/OpenTermUI
```

### Terminal engine (Ghostty source)

OpenTerm vendors Ghostty as a git clone under `native/ghostty/` (gitignored).
Branding and behavior patches are applied in that tree.

```sh
make clone-ghostty    # clone ghostty-org/ghostty at the pinned commit
make terminal         # build GhosttyKit (Zig) + Ghostty.app (Xcode)
make app              # package OpenTerm.app
```

Pin and checksums live in `native/ghostty.lock`. If the source build is unavailable,
fall back to the official Ghostty DMG:

```sh
OPENTERM_USE_DMG=1 make app
# or
GHOSTTY_APP=/path/to/Ghostty.app make app
```

See `docs/native-terminal.md` for runtime layout, signing, and packaging details.

## Commands

Run `ai-help` for the full catalog (supports `--markdown` and `--plain`).

| Area | Commands |
|---|---|
| Agent | `ai-ask`, `ai-connect`, `ai-explain`, `ai-panel`, `ai-workspace` |
| System | `ai-doctor`, `ai-context`, `ai-ui`, `ai-version`, `ai-help` |
| Search & inspect | `ai-rg`, `ai-read`, `ai-ls`, `ai-json`, `ai-tree`, `ai-find`, `ai-grep`, … |
| Development | `ai-test`, `ai-lint`, `ai-cargo`, `ai-pytest`, `ai-tsc`, `ai-go`, `ai-gradle`, … |
| Evidence | `ai-run`, `ai-result`, `ai-expand`, `ai-page`, `ai-export`, `ai-pin`, `ai-forget`, `ai-restore` |
| History | `ai-history`, `ai-search-history`, `ai-clean-history`, `ai-usage` |

RTK-backed commands print a stable evidence reference on stderr:

```text
openterm: result res_ab12... (expand with: ai-expand res_ab12...)
```

Inspect and retrieve without rerunning:

```sh
ai-result res_ab12... --json
ai-expand res_ab12...
ai-explain --last
```

## Local evidence store

Metadata uses SQLite WAL. Evidence blobs use XChaCha20-Poly1305 with a private
mode-0600 local key. Set `OPENTERM_DATA_DIR` to override the default data path.
History retention scoring is documented in `docs/synaptic-retention.md`.

The launcher respects `NO_COLOR` and `OPENTERM_REDUCED_MOTION`. Run
`openterm mascot` to preview the canonical mascot animation.

Inside the native app, zsh and Bash retain the user's existing configuration
and gain an OpenTerm layer: welcome banner, dynamic window title, `ai-cd`, and
live Git context. Run `OpenTermUI` or `ai-ui` to edit theme, font, colors,
opacity, glass blur, padding, and titlebar with live preview.

Source the integration to enable `ai-cd` in a parent shell:

```sh
source /path/to/OpenTerm/shell/openterm.zsh
# or
source /path/to/OpenTerm/shell/openterm.bash
```

## LLM providers

`ai-ask` and `ai-explain` are read-only advisers. Codex is the default provider
and runs through `codex exec` with ephemeral, read-only settings.

After `ai-connect`, stored provider and model defaults apply automatically.
Identical `ai-ask` prompts reuse a local 24-hour LRU cache; OpenAI requests also
send a stable prompt-cache key so the gateway can reuse prefixes.

```sh
ai-panel
ai-ask "what is using the most disk space here?"
ai-ask --workspace "summarize this repo"
ai-ask --provider openai --model gpt-4o-mini "explain df -h"
ai-ask --provider ollama --model llama3.2 "explain df -h"
```

| Provider | Credential |
|---|---|
| Codex CLI | Existing `codex login` session |
| OpenAI Responses API | `ai-connect` or `OPENAI_API_KEY` |
| Claude Messages API | `ai-connect` or `ANTHROPIC_API_KEY` |
| Mistral Chat Completions | `ai-connect` or `MISTRAL_API_KEY` |
| Ollama | Running local runtime (`OLLAMA_HOST`, default `http://127.0.0.1:11434`) |
| LM Studio | Running local runtime (`LMSTUDIO_HOST`, default `http://127.0.0.1:1234`) |

`ai-doctor` reports configured / not configured only — never prints secrets.
Direct-provider keys move to Keychain and Secret Service before the next
distributable alpha.
Provider architecture and safety rules: `docs/provider-architecture.md`.

Enable the repository pre-commit hook once:

```sh
git config core.hooksPath .githooks
```

## Repository map

```text
src/                  OpenTerm CLI, providers, evidence store, ai-connect
tests/                CLI behavior tests
docs/                 Architecture, native terminal, provider contracts
shell/                zsh and Bash parent-shell integrations
native/               Ghostty lockfile, macOS launcher, config, patches
native/ghostty/       Ghostty source clone (gitignored; make clone-ghostty)
scripts/              Build, verify, and packaging scripts
.cursor/rules/        Canonical Pelloux engineering discipline
roadmap.md            Product, architecture, and delivery roadmap
```

Current delivery status is tracked in `docs/implementation-status.md`.

## Engineering rules

The canonical local rules are in `.cursor/rules/pelloux-guidelines.mdc`. Production Rust files target 300 lines and fail above 400; functions target 25 lines; APIs accept at most four parameters. `make verify` enforces formatting, compilation, Clippy, tests, and file-size limits.

All new user-facing OpenTerm tool executables must begin with `ai-`. The `openterm` executable is reserved for launching and controlling the application.

## License

OpenTerm is licensed under Apache-2.0. Third-party and mascot provenance is
recorded in `THIRD_PARTY_NOTICES.md` and `design/mascot/README.md`.
