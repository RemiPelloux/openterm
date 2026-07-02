# OpenTerm mascot: Byte

Byte is OpenTerm's original mascot: a small terminal window with an antenna,
a title bar, dot eyes, and a smile, standing on two feet. It was drawn for
OpenTerm and is released under the repository's Apache-2.0 license.

This directory holds the canonical terminal artwork (`art.ts`) and motion
timing (`animation.ts`). `preview.txt` shows the default poses.

## Canonical behavior

- Default, look-left, look-right, and arms-up poses.
- Click chooses randomly between jump/wave and look-around.
- Each frame lasts 60 ms.
- Jump/wave: crouch, spring with arms up, settle, then repeat once.
- Look-around: look right for five frames, left for five frames, then settle.
- Ignore clicks while an animation is active.
- Disable nonessential animation when reduced motion is enabled.
- Keep a fixed layout footprint so movement never shifts surrounding UI.
- Use the Apple Terminal eye fallback where required for glyph compatibility.

`art.ts` and `animation.ts` are renderer-neutral canonical data. The future TUI and native GUI adapters must render from these files so the mascot does not drift between surfaces.

