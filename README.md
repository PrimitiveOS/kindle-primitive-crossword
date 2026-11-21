<div align="center">

<img src="icon.png" alt="Crossword icon" width="110">

# Crossword

**Mini crosswords and your puzzles**

[![CI](https://img.shields.io/github/actions/workflow/status/PrimitiveOS/kindle-primitive-crossword/ci.yml?branch=main&label=CI)](https://github.com/PrimitiveOS/kindle-primitive-crossword/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/PrimitiveOS/kindle-primitive-crossword?label=release)](https://github.com/PrimitiveOS/kindle-primitive-crossword/releases)
[![Tag](https://img.shields.io/github/v/tag/PrimitiveOS/kindle-primitive-crossword?label=tag)](https://github.com/PrimitiveOS/kindle-primitive-crossword/tags)
[![Licence: PolyForm Noncommercial 1.0.0](https://img.shields.io/badge/licence-PolyForm%20Noncommercial%201.0.0-blue)](LICENSE)
[![Stars](https://img.shields.io/github/stars/PrimitiveOS/kindle-primitive-crossword?style=social)](https://github.com/PrimitiveOS/kindle-primitive-crossword/stargazers)
[![Store downloads](https://img.shields.io/endpoint?url=https%3A%2F%2Fprimitiveos.github.io%2Fkindle-primitive-catalog%2Fv1%2Fbadges%2Fprimitive-crossword.json)](https://primitiveos.github.io/kindle-primitive-catalog/v1/index.json)

[**Download**](https://github.com/PrimitiveOS/kindle-primitive-crossword/releases/latest) ·
[**Install guide**](INSTALL.md) ·
[**Changelog**](CHANGELOG.md)

</div>

---

Play original mini crosswords or open an unlocked .puz file from your Kindle. Read clues, type answers, check mistakes, and reveal answers one clue at a time.

## Quick start

After setting up PrimitiveOS, open **Store** from Primitive Launcher. Choose **Crossword** under **Apps**,
tap **Get**, and open it from the Launcher grid. The [install guide](INSTALL.md) also covers the standalone USB package.

## Screenshots

<table>
<tr>
<td align="center"><img src="docs/images/crossword.png" width="260" alt="Crossword grid and selected clue in light mode"><br><sub>Start an original mini</sub></td>
<td align="center"><img src="docs/images/crossword-clues.png" width="260" alt="Across and Down clues in the clue list"><br><sub>Browse the clue list</sub></td>
<td align="center"><img src="docs/images/crossword-write-dark.png" width="260" alt="Crossword grid and letter keyboard in dark mode"><br><sub>Enter a word in dark mode</sub></td>
</tr>
</table>

<sub>Mac simulation captures at 1264 × 1680 pixels.</sub>

## Highlights

- **Two original puzzles.** Start with “By the water” or “At the desk.”
- **Check and Reveal.** Check highlights mistakes. Reveal fills the selected answer after confirmation.
- **Import your own puzzles.** Copy a standard unlocked `.puz` with single-letter cells to `documents/`, then choose **Puzzles → Import .puz**.

## Develop locally

Keep `doannt-kindle-tools` and `doannt-kindle-launcher` beside this checkout, then run the following from this repo.

```sh
cargo test --offline --locked --workspace
cargo run --offline --locked -p crossword-desktop
```

The desktop target writes previews to `build/previews/`. The desktop simulation exercises entry, clue selection, and reveal.
On an ARMHF Kindle Oasis 3 running firmware 5.18.2, Crossword opened and closed the Clues list, leaving the
original puzzle unchanged, and returned to Launcher. See the [device notes](docs/device-notes.md) for the
verification checklist.

## Status

| Device | Status |
|---|---|
| Kindle Oasis 3 (1264 × 1680, firmware 5.18.2) | Tested on hardware |

**Current version: 0.1.1.** Changes are in [CHANGELOG.md](CHANGELOG.md).

## Downloads

The badge at the top shows the number of Store downloads. It counts GitHub downloads of this app's release packages, across all versions and architectures, and is refreshed daily from the [Store catalog](https://primitiveos.github.io/kindle-primitive-catalog/v1/index.json).

## Documentation

[Install guide](INSTALL.md) · [Changelog](CHANGELOG.md) · [Device notes](docs/device-notes.md)

## Licence

Copyright (C) 2026 Doan Nguyen.

Code is licensed under [PolyForm Noncommercial 1.0.0](LICENSE). The original documentation, pictures, and bundled puzzles are licensed under [CC BY-NC-SA 4.0](LICENSE-DOCS). The `.puz` reader follows the [puzpy format reference](https://github.com/alexdej/puzpy/blob/master/FileFormat.md).
