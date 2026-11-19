use std::path::{Path, PathBuf};

fn main() -> Result<(), String> {
    let dir = Path::new("/mnt/us/crossword");
    let documents = PathBuf::from("/mnt/us/documents");
    primitive_app::run(
        crossword_render::View::load(dir.join("puzzle.json"), documents),
        dir,
    )
}
