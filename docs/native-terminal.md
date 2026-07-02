# Native terminal alpha

## What ships

`make app` creates:

- `dist/OpenTerm.app`
- `dist/OpenTerm-0.1.0-alpha-macos-arm64.dmg`

The app uses Ghostty 1.3.1 for standards-compliant VT emulation, PTYs, Metal
rendering, CoreText, tabs, splits, scrollback, selection, and shell integration.
OpenTerm contributes the app identity, native launcher, configuration, mascot
icon, evidence engine, provider adapters, bundled RTK, and `ai-*` tools.

The app also bundles two OpenTerm-owned native surfaces:

- an accessory mascot animation shown while the terminal starts;
- `OpenTermUI`, a SwiftUI appearance editor with live preview and config reload.

## Runtime layout

The native launcher prepends this directory to every terminal shell:

```text
OpenTerm.app/Contents/Resources/openterm/bin
```

It contains the OpenTerm multicall binary, all generated `ai-*` links, and the
pinned RTK binary. The native launcher also routes zsh and Bash through a thin
integration layer that first loads the user’s original startup files. User state
is isolated at:

```text
~/Library/Application Support/OpenTerm/
```

The first launch copies `config.ghostty` there so the user can customize it
without modifying the application bundle.

## Reproducibility and signing

Inputs and SHA-256 values are pinned in `native/ghostty.lock`. The build verifies
Ghostty source, its official DMG, Zig, and bundled RTK before packaging.

The current app is ad-hoc signed and intended for local development. Public
distribution still requires an Apple Developer ID, hardened-runtime entitlement
review, notarization, stapling, update signing, and release-channel provenance.

The packager caches the verified pristine terminal payload before replacing any
existing app. In restricted environments where disk images cannot be mounted,
mount the official Ghostty DMG normally and run:

```sh
GHOSTTY_APP=/Volumes/Ghostty/Ghostty.app make app
```

The previous OpenTerm alpha DMG can also seed the cache:

```sh
GHOSTTY_APP=/Volumes/OpenTerm/OpenTerm.app make app
```

## Verified behavior

- Deep code-signature verification passes.
- Bundle identifier resolves as `com.openpro.openterm`.
- A real GUI window and PTY-backed login shell remain alive after launch.
- AppleScript can create tabs and terminal surfaces.
- A command executed inside an OpenTerm tab resolves bundled `ai-history`,
  `openterm 0.1.0`, and `rtk 0.43.0`.
- The app loads the macOS Metal driver.

## Known boundary

This alpha wraps the official stable terminal payload. OpenTerm-specific command
blocks, AI sidecar panels, daemon-owned persistent sessions, and direct
libghostty integration are subsequent native slices. Automatic upstream updates
are disabled so Ghostty cannot replace the OpenTerm bundle unexpectedly.
