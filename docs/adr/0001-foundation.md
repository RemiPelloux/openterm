# ADR 0001: OpenTerm foundation

Status: accepted — 2026-07-02

## Decision

OpenTerm uses a shared Rust control plane, native platform clients, a pinned
Ghostty integration, and an RTK-derived deterministic compression layer.

The first implementation slice stays in one Cargo package with a workspace
root. Crates split only after a second real consumer establishes an ownership
boundary. This preserves the planned architecture without adding empty crates.

Managed commands execute once. Their exact observed stdout and stderr are
encrypted locally and indexed in SQLite. Until RTK filters are extracted,
RTK-backed records are explicitly marked `rtk-filtered-output`; they are not
misrepresented as the underlying tool's raw stream.

## Rejected alternatives

- Building VT emulation and rendering from scratch duplicates mature work.
- Making tmux mandatory constrains the UI and protocol architecture.
- Adding storage independently to every wrapper duplicates policy and errors.
