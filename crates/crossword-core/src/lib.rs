//! Original mini crosswords, across/down navigation, and standard .puz parsing.

use serde::{Deserialize, Serialize};

pub const MAX_FILE_SIZE: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Across,
    Down,
}

impl Direction {
    pub fn name(self) -> &'static str {
        match self {
            Self::Across => "Across",
            Self::Down => "Down",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Clue {
    pub number: usize,
    pub direction: Direction,
    pub text: String,
    pub squares: Vec<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub title: String,
    pub author: String,
    pub width: usize,
    pub height: usize,
    pub clues: Vec<Clue>,
    solution: Vec<u8>,
}

impl Puzzle {
    fn build(
        title: String,
        author: String,
        width: usize,
        height: usize,
        solution: Vec<u8>,
        text: Vec<String>,
    ) -> Result<Self, &'static str> {
        if !(2..=25).contains(&width)
            || !(2..=25).contains(&height)
            || solution.len() != width * height
            || !solution
                .iter()
                .all(|&c| c == b'.' || c.is_ascii_uppercase())
        {
            return Err("The puzzle has an unsupported grid.");
        }
        let mut clues = Vec::new();
        let mut number = 0;
        let mut text = text.into_iter();
        let open = |i| solution.get(i).is_some_and(|&c| c != b'.');
        for square in 0..solution.len() {
            if !open(square) {
                continue;
            }
            let column = square % width;
            let across =
                (column == 0 || !open(square - 1)) && column + 1 < width && open(square + 1);
            let down = (square < width || !open(square - width)) && open(square + width);
            if across || down {
                number += 1;
            }
            if across {
                let squares = (square..square + width - column)
                    .take_while(|&index| open(index))
                    .collect();
                let text = text.next().ok_or("The puzzle has missing clues.")?;
                clues.push(Clue {
                    number,
                    direction: Direction::Across,
                    text,
                    squares,
                });
            }
            if down {
                let squares = (square..solution.len())
                    .step_by(width)
                    .take_while(|&index| open(index))
                    .collect();
                let text = text.next().ok_or("The puzzle has missing clues.")?;
                clues.push(Clue {
                    number,
                    direction: Direction::Down,
                    text,
                    squares,
                });
            }
        }
        if text.next().is_some()
            || clues.is_empty()
            || clues.iter().any(|clue| clue.text.trim().is_empty())
        {
            return Err("The puzzle's clues do not match its grid.");
        }
        let mut used = vec![false; solution.len()];
        for clue in &clues {
            for &square in &clue.squares {
                used[square] = true;
            }
        }
        if solution
            .iter()
            .enumerate()
            .any(|(i, &c)| c != b'.' && !used[i])
        {
            return Err("The puzzle has a letter outside any clue.");
        }
        Ok(Self {
            title,
            author,
            width,
            height,
            clues,
            solution,
        })
    }

    pub fn number(&self, index: usize) -> Option<usize> {
        self.clues
            .iter()
            .find(|clue| clue.squares.first() == Some(&index))
            .map(|clue| clue.number)
    }

    pub fn black_square(&self, index: usize) -> bool {
        self.solution.get(index).is_none_or(|&cell| cell == b'.')
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    pub puzzle: Puzzle,
    filled: Vec<u8>,
    revealed: Vec<bool>,
    active: usize,
    square: usize,
}

impl Game {
    pub fn new(puzzle: Puzzle) -> Self {
        let size = puzzle.solution.len();
        let square = puzzle.clues[0].squares[0];
        Self {
            puzzle,
            filled: vec![0; size],
            revealed: vec![false; size],
            active: 0,
            square,
        }
    }

    pub fn is_valid(&self) -> bool {
        let Ok(rebuilt) = Puzzle::build(
            self.puzzle.title.clone(),
            self.puzzle.author.clone(),
            self.puzzle.width,
            self.puzzle.height,
            self.puzzle.solution.clone(),
            self.puzzle
                .clues
                .iter()
                .map(|clue| clue.text.clone())
                .collect(),
        ) else {
            return false;
        };
        rebuilt.clues.iter().zip(&self.puzzle.clues).all(|(a, b)| {
            a.number == b.number && a.direction == b.direction && a.squares == b.squares
        }) && self
            .puzzle
            .clues
            .get(self.active)
            .is_some_and(|clue| clue.squares.contains(&self.square))
            && self.filled.len() == self.puzzle.solution.len()
            && self.revealed.len() == self.filled.len()
            && self
                .filled
                .iter()
                .zip(&self.puzzle.solution)
                .zip(&self.revealed)
                .all(|((&entry, &solution), &revealed)| {
                    if revealed {
                        entry == solution && solution != b'.'
                    } else {
                        entry == 0 || (solution != b'.' && entry.is_ascii_uppercase())
                    }
                })
    }

    pub fn clue(&self) -> (usize, &Clue) {
        (self.active, &self.puzzle.clues[self.active])
    }

    pub fn square(&self) -> usize {
        self.square
    }

    pub fn entered(&self, index: usize) -> Option<char> {
        self.filled
            .get(index)
            .copied()
            .filter(|&c| c != 0)
            .map(char::from)
    }

    pub fn revealed(&self, index: usize) -> bool {
        self.revealed[index]
    }

    pub fn mistake(&self, index: usize) -> bool {
        self.filled[index] != 0 && self.filled[index] != self.puzzle.solution[index]
    }

    pub fn mistakes(&self) -> usize {
        (0..self.filled.len())
            .filter(|&index| self.mistake(index))
            .count()
    }

    pub fn complete(&self) -> bool {
        self.puzzle
            .solution
            .iter()
            .zip(&self.filled)
            .all(|(&answer, &entry)| answer == b'.' || answer == entry)
    }

    pub fn select_clue(&mut self, index: usize) -> bool {
        if index == self.active {
            return false;
        }
        let Some(clue) = self.puzzle.clues.get(index) else {
            return false;
        };
        self.active = index;
        self.square = clue
            .squares
            .iter()
            .copied()
            .find(|&i| self.filled[i] == 0)
            .unwrap_or(clue.squares[0]);
        true
    }

    pub fn select_square(&mut self, square: usize) -> bool {
        let prior = &self.puzzle.clues[self.active];
        let preferred = self.puzzle.clues.iter().position(|clue| {
            clue.squares.contains(&square)
                && (clue.direction == prior.direction) != (square == self.square)
        });
        let alternate = self
            .puzzle
            .clues
            .iter()
            .position(|clue| clue.squares.contains(&square));
        let Some(clue) = preferred.or(alternate) else {
            return false;
        };
        let changed = (self.active, self.square) != (clue, square);
        (self.active, self.square) = (clue, square);
        changed
    }

    pub fn move_cursor(&mut self, steps: i32) -> bool {
        let clue = &self.puzzle.clues[self.active];
        let position = clue
            .squares
            .iter()
            .position(|&square| square == self.square)
            .unwrap_or(0);
        let next = (position as i32 + steps).clamp(0, clue.squares.len() as i32 - 1) as usize;
        self.square = clue.squares[next];
        next != position
    }

    pub fn enter(&mut self, key: char) -> bool {
        if !key.is_ascii_alphabetic() {
            return false;
        }
        let value = key.to_ascii_uppercase() as u8;
        let old = self.filled[self.square];
        if !self.revealed[self.square] {
            self.filled[self.square] = value;
        }
        let changed = old != self.filled[self.square];
        self.move_cursor(1) || changed
    }

    pub fn delete(&mut self) -> bool {
        let moved = self.filled[self.square] == 0 && self.move_cursor(-1);
        if self.revealed[self.square] || self.filled[self.square] == 0 {
            return moved;
        }
        self.filled[self.square] = 0;
        true
    }

    pub fn reveal_clue(&mut self) {
        for &square in &self.puzzle.clues[self.active].squares {
            self.filled[square] = self.puzzle.solution[square];
            self.revealed[square] = true;
        }
    }
}

pub const BUNDLED: [&str; 2] = ["By the water", "At the desk"];

pub fn bundled(index: usize) -> Game {
    let (grid, clues): (&[&str], &[&str]) = match index {
        1 => (
            &[
                ".PAPER...",
                ".R.......",
                ".INK.....",
                ".N.......",
                ".TABLE...",
                ".....D...",
                ".....I...",
                "...NOTES.",
            ],
            &[
                "A surface for writing",
                "Put marks on paper with a machine",
                "Liquid in a pen",
                "Furniture with a flat top",
                "Revise or correct copy",
                "Brief written reminders",
            ],
        ),
        _ => (
            &[
                "..OWL....",
                "..C......",
                "..ELK....",
                "..A......",
                "CANOE....",
                "....G....",
                "....R....",
                "....E....",
                "....TIDE.",
            ],
            &[
                "Bird heard at night",
                "A large body of salt water",
                "A deer with broad antlers",
                "A narrow boat moved with paddles",
                "A wading bird with long legs",
                "The sea's regular rise and fall",
            ],
        ),
    };
    let puzzle = Puzzle::build(
        BUNDLED[index.min(1)].to_string(),
        "Primitive AI".to_string(),
        grid[0].len(),
        grid.len(),
        grid.join("").into_bytes(),
        clues.iter().map(ToString::to_string).collect(),
    )
    .expect("valid original crossword");
    Game::new(puzzle)
}

pub fn parse_puz(data: &[u8]) -> Result<Game, &'static str> {
    if data.len() > MAX_FILE_SIZE || data.get(2..14) != Some(b"ACROSS&DOWN\0") {
        return Err("This file is not a supported .puz crossword.");
    }
    let header = data.get(..52).ok_or("The .puz header is incomplete.")?;
    let width = usize::from(header[44]);
    let height = usize::from(header[45]);
    let count = usize::from(u16::from_le_bytes([header[46], header[47]]));
    if u16::from_le_bytes([header[48], header[49]]) != 1 || header[50..52] != [0, 0] {
        return Err("Only unlocked, standard crosswords can be opened.");
    }
    let size = width * height;
    let solution = data
        .get(52..52 + size)
        .ok_or("The .puz answer grid is incomplete.")?;
    let fill = data
        .get(52 + size..52 + size * 2)
        .ok_or("The .puz fill grid is incomplete.")?;
    let mut remaining = data
        .get(52 + size * 2..)
        .ok_or("The .puz clues are missing.")?;
    let title = string(&mut remaining)?;
    let author = string(&mut remaining)?;
    let _copyright = string(&mut remaining)?;
    let texts = (0..count)
        .map(|_| string(&mut remaining))
        .collect::<Result<Vec<_>, _>>()?;
    if !remaining.is_empty() {
        string(&mut remaining)?;
    }
    check_extensions(remaining)?;
    let puzzle = Puzzle::build(
        title,
        author,
        width,
        height,
        solution.iter().map(u8::to_ascii_uppercase).collect(),
        texts,
    )?;
    let mut game = Game::new(puzzle);
    for (index, &value) in fill.iter().enumerate() {
        match value {
            b'.' if game.puzzle.black_square(index) => {}
            b'-' if !game.puzzle.black_square(index) => {}
            letter if letter.is_ascii_alphabetic() && !game.puzzle.black_square(index) => {
                game.filled[index] = letter.to_ascii_uppercase()
            }
            _ => return Err("The .puz fill grid does not match the solution."),
        }
    }
    Ok(game)
}

fn string(remaining: &mut &[u8]) -> Result<String, &'static str> {
    let end = remaining
        .iter()
        .position(|&b| b == 0)
        .ok_or("The .puz clues are incomplete.")?;
    let value = decode(&remaining[..end]);
    *remaining = &remaining[end + 1..];
    Ok(value)
}

fn decode(bytes: &[u8]) -> String {
    const WINDOWS: [char; 32] = [
        '€', '�', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '�', 'Ž', '�', '�', '‘',
        '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '�', 'ž', 'Ÿ',
    ];
    if let Ok(utf8) = std::str::from_utf8(bytes) {
        return utf8.into();
    }
    bytes
        .iter()
        .map(|&b| {
            if (128..160).contains(&b) {
                WINDOWS[usize::from(b - 128)]
            } else {
                char::from(b)
            }
        })
        .collect()
}

fn check_extensions(mut detail: &[u8]) -> Result<(), &'static str> {
    while !detail.is_empty() && detail[0] != 0 {
        let header = detail.get(..8).ok_or("A .puz extension is incomplete.")?;
        let size = usize::from(u16::from_le_bytes([header[4], header[5]]));
        let content = detail
            .get(8..8 + size)
            .ok_or("A .puz extension is incomplete.")?;
        if &header[..4] == b"GRBS" && content.iter().any(|&byte| byte != 0) {
            return Err("Rebus crosswords need more than one letter per square.");
        }
        detail = detail
            .get(8 + size + 1..)
            .ok_or("A .puz extension is incomplete.")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_bundled_crossword_can_be_filled_and_revealed() {
        let mut game = bundled(0);
        assert!(game.is_valid());
        assert_eq!(game.clue().1.direction, Direction::Across);
        assert!(game.enter('o'));
        assert!(game.enter('x'));
        assert!(game.enter('l'));
        assert_eq!(game.mistakes(), 1);
        game.reveal_clue();
        assert_eq!(game.mistakes(), 0);
        assert_eq!(game.entered(3), Some('W'));
        for index in 0..game.puzzle.clues.len() {
            game.select_clue(index);
            game.reveal_clue();
        }
        assert!(game.complete());
        assert!(bundled(1).is_valid());
    }

    #[test]
    fn opens_an_original_standard_puz_fixture_and_refuses_locked_or_bad_clues() {
        let mut file = vec![0; 52];
        file[2..14].copy_from_slice(b"ACROSS&DOWN\0");
        file[44..51].copy_from_slice(&[2, 2, 4, 0, 1, 0, 0]);
        file.extend_from_slice(b"IOONI---Original\0Primitive\0\0One Jupiter moon\0That moon again\0Switch position\0Opposite of off\0\0");
        let game = parse_puz(&file).unwrap();
        assert!(game.is_valid());
        assert_eq!(game.entered(0), Some('I'));
        assert_eq!(game.puzzle.clues[1].direction, Direction::Down);
        assert!(parse_puz(&file[..63]).is_err());
        file[50] = 4;
        assert!(parse_puz(&file).is_err());
    }
}
