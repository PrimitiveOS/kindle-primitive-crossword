use std::{
    fs,
    path::{Path, PathBuf},
};

use crossword_core::bundled;
use crossword_render::View;
use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input, Key, Layer, Layout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&out)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let mut app = View::new(bundled(0), None, None);
    write_png(
        &out.join("crossword.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    assert_eq!(app.input(Input::Tap(150.0, 1550.0)), Change::Full);
    let layout = Layout::new(WIDTH, HEIGHT, 4, 5);
    let tap = |target| {
        let (_, rect) = layout
            .keyboard_keys_with(Layer::Upper, "Hide")
            .into_iter()
            .find(|(key, _)| key.key == target)
            .expect("keyboard key");
        Input::Tap(
            rect.x as f32 + rect.w as f32 / 2.0,
            rect.y as f32 + rect.h as f32 / 2.0,
        )
    };
    for letter in ['O', 'X', 'L'] {
        assert_eq!(app.input(tap(Key::Char(letter))), Change::Partial);
    }
    assert_eq!(app.game().mistakes(), 1);
    write_png(
        &out.join("crossword-write-dark.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, true),
    )?;
    assert_eq!(app.input(tap(Key::Go)), Change::Full);
    assert_eq!(app.input(Input::Tap(850.0, 1550.0)), Change::Full);
    write_png(
        &out.join("crossword-reveal.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    assert_eq!(app.input(Input::Tap(770.0, 960.0)), Change::Full);
    assert_eq!(app.game().mistakes(), 0);
    assert_eq!(app.input(Input::Tap(385.0, 1550.0)), Change::Full);
    write_png(
        &out.join("crossword-clues.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    Ok(())
}
