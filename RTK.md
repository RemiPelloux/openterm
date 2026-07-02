# RTK usage

RTK is OpenTerm's current command-output compression backend.

- Prefer `rtk` wrappers for commands it supports when output may be large.
- Use `rtk git`, `rtk grep`, `rtk find`, `rtk read`, and `rtk test` for agent-facing output.
- Use raw commands when exact byte-for-byte output is required.
- Never run both an RTK wrapper and an automatic RTK rewrite for the same command.
- Preserve exit codes and inspect raw failure output when compression omits required evidence.
- Current development baseline: RTK 0.43 or newer until a protocol is pinned.

