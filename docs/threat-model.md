# OpenTerm foundation threat model

## Protected assets

- Command output and workspace paths.
- Provider credentials and authentication sessions.
- File mutations, approvals, and remote-host identity.
- Plugin capabilities and result references.

## Current trust boundaries

- Managed child processes are untrusted producers of bytes.
- RTK is an external executable and must be resolved deliberately from `PATH`.
- The local evidence directory is private; blobs are encrypted with
  XChaCha20-Poly1305 and a mode-0600 key.
- SQLite metadata contains paths and argv but no environment values.
- LLM network calls occur only after explicit provider selection.

## Current limitations

- The evidence key is a private local file, not yet backed by Keychain or
  Secret Service.
- RTK output is captured after RTK filtering until the native extraction lands.
- Command argv may itself contain secrets; secret-flag redaction is required
  before managed execution expands beyond the current read-oriented commands.
- Recoverable trash has no automatic 30-day purge yet.
- On macOS, direct-provider keys live in the Keychain; elsewhere (and when it
  is locked or `OPENTERM_KEYCHAIN=off`) they fall back to a mode-0600 JSON
  compatibility file. Secret Service support on Linux is still pending.

Codex authentication is owned by the installed Codex CLI. OpenTerm must not
read or persist Codex OAuth tokens or embed private authentication constants.

These limitations block a public security claim, not local development.
