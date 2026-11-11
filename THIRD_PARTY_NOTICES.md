# Third-party notices

OpenTerm is licensed under Apache-2.0. Release artifacts must include the full
license texts for every distributed dependency.

## RTK

The CLI currently invokes RTK as an external executable; the macOS alpha bundles
RTK 0.43.0. The planned `openterm-compress` extraction will derive from
[`rtk-ai/rtk`](https://github.com/rtk-ai/rtk), licensed under Apache-2.0.
Derived source must retain RTK copyright and attribution notices.

Current external-source candidate remains
`b52e02b4375f4415a0d8a11d45066e1cf856da80` (`develop`). Bundled RTK is pinned
separately by version and binary SHA-256 in `native/ghostty.lock`.

## Ghostty

The macOS alpha repackages the official Ghostty 1.3.1 universal terminal payload
and will progressively replace the packaging boundary with a maintained fork of
[`ghostty-org/ghostty`](https://github.com/ghostty-org/ghostty), licensed under
MIT. Ghostty copyright and license notices must accompany distributed code.

Pinned release revision:
`332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28` (`v1.3.1`). Source and official
DMG checksums are recorded in `native/ghostty.lock`.
