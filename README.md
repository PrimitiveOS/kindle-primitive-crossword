# Primitive Crossword

Solve original mini crosswords or open your own classic Across Lite `.puz` files. Choose a clue, select a square, and
use the Kindle keyboard to enter letters. **Check** highlights mistakes and **Reveal** fills the selected answer after
confirmation. The open puzzle and its progress are saved.

Place unlocked, standard `.puz` files in `documents/`, then choose **Puzzles → Import .puz**.

## Develop locally

Run `cargo test --offline --workspace` and `cargo run --offline -p crossword-desktop` from this directory with Primitive
Tools and Launcher beside it. The desktop app runs sample entry and reveal interactions and writes previews to
`build/previews`. See [device notes](docs/device-notes.md) for testing and the bundled puzzles' licence.
