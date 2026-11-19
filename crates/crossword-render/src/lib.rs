use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use crossword_core::{bundled, parse_puz, Game, BUNDLED, MAX_FILE_SIZE};
use primitive_app::{App, Change, HEIGHT, WIDTH};
use primitive_ui::{
    back_button, header, keyboard_key, screen_rect, Canvas, Editor, Input, Key, Layer, Layout,
    Rect, TextStyle, BLACK, CAPTION, HAIRLINE, PAPER,
};

const TILE: i32 = 103;
const BOARD_Y: i32 = 245;
const PAGE_ROWS: usize = 7;
const REVEAL_BOX: Rect = Rect::new(240, 565, 784, 490);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Clues,
    Puzzles,
    Import,
    Reveal,
}

pub struct View {
    game: Game,
    data: Option<PathBuf>,
    documents: Option<PathBuf>,
    editing: bool,
    editor: Editor,
    checked: bool,
    panel: Option<Panel>,
    index: usize,
    files: Vec<PathBuf>,
    notice: String,
}

impl View {
    pub fn new(game: Game, data: Option<PathBuf>, documents: Option<PathBuf>) -> Self {
        let mut editor = Editor::new(1);
        editor.layer = Layer::Upper;
        Self {
            game,
            data,
            documents,
            editing: false,
            editor,
            checked: false,
            panel: None,
            index: 0,
            files: Vec::new(),
            notice: String::new(),
        }
    }

    pub fn load(data: PathBuf, documents: PathBuf) -> Self {
        let game = fs::read(&data)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Game>(&bytes).ok())
            .filter(Game::is_valid)
            .unwrap_or_else(|| bundled(0));
        Self::new(game, Some(data), Some(documents))
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    fn record(&mut self) {
        if self.save().is_err() {
            self.notice = "Could not save this crossword.".into();
        }
    }

    fn replace(&mut self, game: Game) {
        self.game = game;
        self.panel = None;
        self.editing = false;
        self.checked = false;
        self.notice.clear();
        self.record();
    }

    fn show(&mut self, panel: Panel) {
        self.panel = Some(panel);
        self.index = 0;
        self.notice.clear();
        if panel == Panel::Import {
            self.files.clear();
            if let Some(documents) = self.documents.as_deref() {
                match primitive_app::with_storage(|| Ok(local_puzzles(documents))) {
                    Ok(files) => self.files = files,
                    Err(error) => self.notice = format!("Could not open Documents. {error}"),
                }
            }
        }
    }

    fn import(&mut self, index: usize) -> Change {
        let Some(path) = self.files.get(index) else {
            return Change::None;
        };
        let contents = primitive_app::with_storage(|| {
            let mut bytes = Vec::new();
            File::open(path)?
                .take((MAX_FILE_SIZE + 1) as u64)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        });
        let result = contents
            .as_deref()
            .map_err(|_| "Could not read the puzzle.")
            .and_then(parse_puz);
        match result {
            Ok(game) => {
                self.replace(game);
                Change::Full
            }
            Err(message) => {
                self.notice = message.into();
                Change::Partial
            }
        }
    }

    fn total(&self, panel: Panel) -> usize {
        match panel {
            Panel::Clues => self.game.puzzle.clues.len(),
            Panel::Puzzles => BUNDLED.len(),
            Panel::Import => self.files.len(),
            Panel::Reveal => 0,
        }
    }

    fn draw_panel(&self, canvas: &mut Canvas, panel: Panel) {
        let w = canvas.bounds().w;
        let (title, description) = match panel {
            Panel::Clues => ("Clues", "Choose a clue to write its answer."),
            Panel::Puzzles => (
                "Puzzles",
                "Choose a puzzle to replace the one you have open.",
            ),
            Panel::Import => (
                "Open a puzzle",
                "Put .puz files in the Kindle's documents folder.",
            ),
            Panel::Reveal => return,
        };
        header(canvas, title, "Back");
        canvas.text(screen_rect(w, 40, 165, 1184, 78), description, CAPTION);
        if panel == Panel::Import && self.files.is_empty() {
            canvas.text(
                screen_rect(w, 40, 370, 1184, 90),
                "No .puz files found.",
                CAPTION.center(),
            );
        }
        if !self.notice.is_empty() {
            canvas.text(screen_rect(w, 40, 230, 1184, 69), &self.notice, CAPTION);
        }
        let total = self.total(panel);
        for (row, index) in (self.index..total).take(PAGE_ROWS).enumerate() {
            let bounds = list_row(w, row);
            let label = match panel {
                Panel::Clues => {
                    let clue = &self.game.puzzle.clues[index];
                    format!("{} {} · {}", clue.number, clue.direction.name(), clue.text)
                }
                Panel::Puzzles => BUNDLED[index].to_owned(),
                Panel::Import => self.files[index]
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                Panel::Reveal => String::new(),
            };
            let pressed = canvas.control_pressed(bounds, false);
            canvas.panel(bounds, 22, 2, if pressed { BLACK } else { PAPER });
            canvas.text(
                bounds.inset(canvas.px(23)),
                &label,
                TextStyle::new(33)
                    .lines(2)
                    .ink(if pressed { PAPER } else { BLACK }),
            );
        }
        if self.index > 0 {
            canvas.button(page_button(w, 0), "Previous", false, false);
        }
        if self.index + PAGE_ROWS < total {
            canvas.button(page_button(w, 1), "Next", false, false);
        }
        if panel == Panel::Puzzles {
            canvas.button(page_button(w, 2), "Import .puz", false, false);
        }
    }

    fn viewport(&self) -> (usize, usize, usize, usize) {
        let limit = if self.editing { 6 } else { 10 };
        let (width, height) = (self.game.puzzle.width, self.game.puzzle.height);
        let (row, column) = (self.game.square() / width, self.game.square() % width);
        let top = (row / limit * limit).min(height.saturating_sub(limit));
        let left = (column / limit * limit).min(width.saturating_sub(limit));
        (top, left, height.min(limit), width.min(limit))
    }

    fn draw_board(&self, canvas: &mut Canvas) {
        let w = canvas.bounds().w;
        let (_, clue) = self.game.clue();
        let (top, left, rows, columns) = self.viewport();
        let board_active = self.panel != Some(Panel::Reveal);
        for r in 0..rows {
            for c in 0..columns {
                let square = (r + top) * self.game.puzzle.width + c + left;
                let bounds = board_cell(w, columns, r, c);
                let black = self.game.puzzle.black_square(square);
                let pressed = board_active && !black && canvas.control_pressed(bounds, false);
                let selected = (self.game.square() == square) ^ pressed;
                let in_clue = clue.squares.contains(&square);
                let revealed = self.game.revealed(square);
                let fill = if black || selected {
                    BLACK
                } else if in_clue {
                    224
                } else {
                    PAPER
                };
                let ink = if selected { PAPER } else { BLACK };
                canvas.fill(bounds, if black || selected { BLACK } else { HAIRLINE });
                canvas.fill(bounds.inset(canvas.px(2)), fill);
                if let Some(number) = self.game.puzzle.number(square) {
                    canvas.text(
                        Rect::new(
                            bounds.x + canvas.px(5),
                            bounds.y + canvas.px(3),
                            canvas.px(44),
                            canvas.px(29),
                        ),
                        &number.to_string(),
                        TextStyle::new(22).ink(ink),
                    );
                }
                if let Some(letter) = self.game.entered(square) {
                    let text = Rect::new(bounds.x, bounds.y + canvas.px(20), bounds.w, bounds.h);
                    let style = TextStyle::new(57).bold().center().ink(ink);
                    canvas.text(text, &letter.to_string(), style);
                }
                let badge = if self.checked && self.game.mistake(square) {
                    "×"
                } else if revealed {
                    "•"
                } else {
                    ""
                };
                if !badge.is_empty() {
                    canvas.text(
                        Rect::new(
                            bounds.right() - canvas.px(31),
                            bounds.y + canvas.px(3),
                            canvas.px(25),
                            canvas.px(25),
                        ),
                        badge,
                        TextStyle::new(22).bold().center().ink(ink),
                    );
                }
            }
        }
    }

    fn page(&mut self, panel: Panel, direction: i32) -> Change {
        let page = self.index / PAGE_ROWS;
        let last = self.total(panel).saturating_sub(1) / PAGE_ROWS;
        let offset = (page as i32 + direction).clamp(0, last as i32) as usize * PAGE_ROWS;
        if offset == self.index {
            return Change::None;
        }
        self.index = offset;
        Change::Full
    }

    fn list_input(&mut self, panel: Panel, input: Input) -> Change {
        let w = WIDTH as i32;
        let (x, y) = match input {
            Input::Tap(x, y) => (x, y),
            Input::PageForward => return self.page(panel, 1),
            Input::PageBack => return self.page(panel, -1),
            _ => return Change::None,
        };
        if back_button(w).contains(x, y) {
            self.panel = if panel == Panel::Import {
                Some(Panel::Puzzles)
            } else {
                None
            };
            self.index = 0;
            self.notice.clear();
            return Change::Full;
        }
        if panel == Panel::Puzzles && page_button(w, 2).contains(x, y) {
            self.show(Panel::Import);
            return Change::Full;
        }
        for (offset, direction) in [(0, -1), (1, 1)] {
            if page_button(w, offset).contains(x, y) {
                return self.page(panel, direction);
            }
        }
        for index in self.index..(self.index + PAGE_ROWS).min(self.total(panel)) {
            if list_row(w, index - self.index).contains(x, y) {
                match panel {
                    Panel::Clues => {
                        self.game.select_clue(index);
                        self.panel = None;
                        self.editing = true;
                    }
                    Panel::Puzzles => self.replace(bundled(index)),
                    Panel::Import => return self.import(index),
                    Panel::Reveal => return Change::None,
                }
                return Change::Full;
            }
        }
        Change::None
    }

    fn type_key(&mut self, key: Key) -> Change {
        let change = match key {
            Key::Go => {
                self.editing = false;
                return Change::Full;
            }
            Key::Char(letter) => self.game.enter(letter),
            Key::Delete => self.game.delete(),
            Key::Left => self.game.move_cursor(-1),
            Key::Right => self.game.move_cursor(1),
            Key::Shift => {
                self.editor.layer = if self.editor.layer == Layer::Upper {
                    Layer::Lower
                } else {
                    Layer::Upper
                };
                return Change::Partial;
            }
            Key::Layer(layer) => {
                self.editor.layer = layer;
                return Change::Partial;
            }
            Key::Space => false,
        };
        if change {
            self.record();
            Change::Partial
        } else {
            Change::None
        }
    }
}

fn local_puzzles(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_file())
            && path
                .extension()
                .is_some_and(|suffix| suffix.eq_ignore_ascii_case("puz"))
        {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn board_cell(w: i32, columns: usize, row: usize, column: usize) -> Rect {
    let left = (1264 - columns as i32 * TILE) / 2;
    screen_rect(
        w,
        left + column as i32 * TILE,
        BOARD_Y + row as i32 * TILE,
        TILE,
        TILE,
    )
}

fn list_row(w: i32, row: usize) -> Rect {
    screen_rect(w, 40, 300 + row as i32 * 132, 1184, 116)
}

fn action(w: i32, index: i32) -> Rect {
    screen_rect(w, 40 + index * 239, 1509, 225, 100)
}

fn page_button(w: i32, index: i32) -> Rect {
    screen_rect(w, 40 + index * 399, 1509, 385, 100)
}

fn popup_button(w: i32, index: i32) -> Rect {
    screen_rect(w, 275 + index * 367, 909, 345, 100)
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        if let Some(panel) = self.panel.filter(|&panel| panel != Panel::Reveal) {
            self.draw_panel(canvas, panel);
            return;
        }
        let w = canvas.bounds().w;
        header(canvas, "Crossword", "Done");
        let (_, clue) = self.game.clue();
        let text = format!("{} {} · {}", clue.number, clue.direction.name(), clue.text);
        canvas.text(
            screen_rect(w, 40, 150, 1184, 92),
            &text,
            TextStyle::new(34).lines(2),
        );
        self.draw_board(canvas);
        if self.editing {
            canvas.keyboard(self.editor.layer, "Hide", None);
        } else {
            let status = if !self.notice.is_empty() {
                self.notice.clone()
            } else if self.game.complete() {
                format!("{} · Puzzle complete.", self.game.puzzle.title)
            } else if self.checked {
                let mistakes = self.game.mistakes();
                match mistakes {
                    0 => "No mistakes so far.".into(),
                    1 => "One mistake is marked.".into(),
                    _ => format!("{mistakes} mistakes are marked."),
                }
            } else {
                format!("{} · {}", self.game.puzzle.title, self.game.puzzle.author)
            };
            canvas.text(
                screen_rect(w, 40, 1352, 1184, 90),
                &status,
                CAPTION.center().lines(2),
            );
            for (index, label) in [
                (0, "Write"),
                (1, "Clues"),
                (2, "Check"),
                (3, "Reveal"),
                (4, "Puzzles"),
            ] {
                canvas.button(action(w, index), label, index == 0, false);
            }
        }
        if self.panel == Some(Panel::Reveal) {
            canvas.popup(screen_rect(
                w,
                REVEAL_BOX.x,
                REVEAL_BOX.y,
                REVEAL_BOX.w,
                REVEAL_BOX.h,
            ));
            canvas.text(
                screen_rect(w, 280, 640, 706, 90),
                "Reveal this clue?",
                TextStyle::new(50).bold().center(),
            );
            canvas.text(
                screen_rect(w, 280, 782, 706, 70),
                "Its answer will fill the squares.",
                CAPTION.center(),
            );
            canvas.button(popup_button(w, 0), "Cancel", false, false);
            canvas.button(popup_button(w, 1), "Reveal", true, false);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        let w = WIDTH as i32;
        if let Some(panel) = self.panel.filter(|&panel| panel != Panel::Reveal) {
            return self.list_input(panel, input);
        }
        if self.panel != Some(Panel::Reveal)
            && matches!(input, Input::PageForward | Input::PageBack)
        {
            let (active, _) = self.game.clue();
            let next = if matches!(input, Input::PageForward) {
                active + 1
            } else {
                active.saturating_sub(1)
            };
            return if self.game.select_clue(next) {
                Change::Full
            } else {
                Change::None
            };
        }
        let Input::Tap(x, y) = input else {
            return Change::None;
        };
        if self.panel == Some(Panel::Reveal) {
            if popup_button(w, 1).contains(x, y) {
                self.game.reveal_clue();
                self.panel = None;
                self.record();
                return Change::Full;
            }
            if popup_button(w, 0).contains(x, y)
                || !screen_rect(w, REVEAL_BOX.x, REVEAL_BOX.y, REVEAL_BOX.w, REVEAL_BOX.h)
                    .contains(x, y)
            {
                self.panel = None;
                return Change::Full;
            }
            return Change::None;
        }
        if back_button(w).contains(x, y) {
            return Change::Exit;
        }
        if self.editing {
            let layout = Layout::new(WIDTH, HEIGHT, 4, 5);
            if let Some(key) = keyboard_key(&layout, &self.editor, "Hide", x, y) {
                return self.type_key(key);
            }
        } else {
            if action(w, 0).contains(x, y) {
                self.editing = true;
                return Change::Full;
            }
            if action(w, 2).contains(x, y) {
                self.checked = !self.checked;
                return Change::Partial;
            }
            for (index, panel) in [(1, Panel::Clues), (3, Panel::Reveal), (4, Panel::Puzzles)] {
                if action(w, index).contains(x, y) {
                    self.show(panel);
                    return Change::Full;
                }
            }
        }
        let (top, left, rows, columns) = self.viewport();
        for r in 0..rows {
            for c in 0..columns {
                if board_cell(w, columns, r, c).contains(x, y) {
                    let square = (r + top) * self.game.puzzle.width + c + left;
                    return if self.game.select_square(square) {
                        Change::Partial
                    } else {
                        Change::None
                    };
                }
            }
        }
        Change::None
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(data) = &self.data {
            primitive_app::save(data, &serde_json::to_vec(&self.game)?)?;
        }
        Ok(())
    }
}
