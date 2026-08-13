# OpenTerm implementation status

This file tracks shipped code against `roadmap.md`. A roadmap item is complete only when its implementation and verification exist in the repository.

## Completed

### Project foundation

- Rust single-binary foundation.
- Pelloux size, shape, formatting, lint, and test enforcement.
- CI and pre-commit verification.
- Canonical OpenTerm mascot art and animation data.
- `ai-*` command namespace invariant.
- Dynamic multicall links generated from the Rust command registry.

### LLM advice foundation

- `ai-ask` attaches workspace and shell files, streams markdown in the terminal,
  and applies edits through OpenCode when the model returns an `opencode` prompt.
- Codex CLI one-shot adapter using ephemeral read-only execution.
- OpenAI Responses API adapter, with optional `base_url` / `OPENAI_BASE_URL`.
- Claude Messages API adapter.
- Mistral Chat Completions adapter.
- Bounded API response parsing with redirects disabled.
- Missing-credential diagnostics without secret disclosure.
- Happy and negative parser/CLI tests.

### RTK P0 compatibility slice

- `ai-grep` → `rtk grep`.
- `ai-find` → `rtk find`.
- `ai-tree` → `rtk tree`.
- `ai-read` → `rtk read`.
- `ai-status` → `rtk git status`.
- `ai-diff` → `rtk git diff`.
- `ai-test` → `rtk test`.
- `ai-log` → `rtk log`.
- Safe argv forwarding without shell interpolation.
- Child exit-code and Unix signal-code preservation.
- Command-not-found behavior uses exit code 127.
- Managed wrappers for lint/format, TypeScript, JavaScript tests, Python, Rust,
  Go, Ruby, JVM, and .NET development output.
- 39 total RTK-backed commands with unique-name and route tests.

### Persistent evidence foundation

- Stable `res_*` identifiers for successful RTK process launches.
- SQLite WAL metadata with argv, cwd, timing, exit code, byte counts, and route.
- XChaCha20-Poly1305 encrypted stdout/stderr blobs.
- `ai-result`, `ai-expand`, `ai-page`, `ai-export`, `ai-pin`, and `ai-forget`.
- Recoverable trash instead of immediate evidence deletion.
- `ai-restore` with paired stdout/stderr moves and rollback on partial failure.
- Single RTK execution; evidence retrieval never reruns the managed command.
- Evidence is honestly marked `rtk-filtered-output` until native extraction.
- zsh and Bash `ai-cd` parent-shell integrations.
- `ai-run PROGRAM [ARG ...]` raw managed execution without shell interpolation.

### Smart history and cockpit

- Accessible, color-aware OpenTerm launcher cockpit using the canonical mascot.
- Exact 60 ms mascot jump/wave preview with reduced-motion fallback.
- `ai-history`, `ai-search-history`, `ai-clean-history`, and `ai-usage`.
- Explainable logistic retention score with exponential freshness decay.
- Access counters update when evidence is expanded or paged.
- Cleanup remains suggestion-only and never deletes automatically.
- Versioned SQLite migration for history intelligence fields.

### Native macOS terminal alpha

- Launchable `OpenTerm.app` using the Ghostty 1.3.1 terminal payload.
- Metal renderer, CoreText fonts, PTYs, tabs, splits, scrollback, and native UI.
- OpenTerm bundle identity, menu labels, default theme, and mascot application icon.
- Native Mach-O launcher injects OpenTerm paths before starting the terminal engine.
- All current `ai-*` commands and RTK 0.43.0 are bundled into shell `PATH`.
- Per-user config and evidence data live under Application Support/OpenTerm.
- Automatic Ghostty/Sparkle updates are disabled in the OpenTerm bundle.
- Deep ad-hoc signing, config validation, GUI launch, window, PTY, and bundled-tool smoke tests.
- Reproducible local DMG target through `make app`.

### Native UX and repository context

- Native SwiftUI `OpenTermUI` window with five presets and a live terminal preview.
- Font, size, background, foreground, accent, selection, opacity, blur, padding,
  balanced spacing, and titlebar controls with atomic config writes.
- `ai-ui` and explicit `OpenTermUI` launch paths.
- `ai-panel` opens a non-activating agent side panel so the shell keeps focus.
- `ai-chat` / `openterm chat` is an in-terminal mascot chat with workspace context.
- Prompt top bar shows path, provider, token totals, cache hits, and AI liveness.
- `ai-settings` / `openterm settings` open appearance, AI, and token settings.
- Accessory mascot splash with canonical 60 ms jump/wave frames.
- Automatic zsh/Bash wrapper that preserves existing user startup files.
- `ai-context` human, prompt, and JSON modes based on Git porcelain v2.
- Branch, staged, modified, renamed, untracked, conflicts, ahead/behind, and stashes.
- Git context in terminal title and zsh right prompt; Bash renders it above the prompt.

### Provider hardening

- `ai-connect` delegates Codex authentication to the installed Codex CLI.
- OpenTerm does not parse, copy, transmit, or persist Codex OAuth tokens.
- Direct-provider model discovery remains isolated by transport.

## Current verification

Run:

```sh
make verify
make build
make app
./target/debug/ai-doctor
```

The ordinary CI path does not make paid provider calls. Live API tests remain opt-in.

## Next slice

### Native compression and raw capture

Goal: extract the Apache-2.0 RTK filtering core so OpenTerm can tee the
underlying tool's raw stream before deterministic compression.

Planned acceptance criteria:

- stream output while capturing rather than buffering until process exit;
- preserve stdout/stderr ordering metadata and interrupted partial evidence;
- record RTK/OpenTerm compressor version and selected filter;
- compare native filters against pinned upstream golden fixtures;
- redact secret-bearing argv fields before expanding managed command coverage;
- move evidence and provider keys into Keychain and Secret Service;
- implement opt-in 30-day trash purge.
- migrate and remove legacy plaintext provider secrets.

## Later

- Persistent session daemon and PTYs.
- Shell command-boundary integration.
- Codex app-server rich client.
- Documentation agent.
- OpenTerm-native AI sidecar and semantic command blocks.
- Plugin SDK and marketplace.
- Remote/SSH adapters.

These remain roadmap commitments, not implemented features.
