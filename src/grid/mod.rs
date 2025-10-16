pub mod index;

use std::{
    collections::{HashMap, HashSet},
};

pub use index::{Coordinate, GridIndex};

use crate::{Error, box_array};

/// The amount of columns in a grid.
/// The default is 256.
const GRID_WIDTH: usize = 256;
/// The amount of rows in the grid.
/// The default is 256.
const GRID_HEIGHT: usize = 256;

/// A 2D, fixed size array on the heap.
#[derive(Debug, Clone)]
pub struct Grid<T>(Box<[[T; GRID_HEIGHT]; GRID_WIDTH]>);

impl<T> Grid<T>
where T: Copy {
    /// Constructs a `Grid`.
    pub fn new(filler: T) -> Self {
        Self(box_array![[filler; GRID_HEIGHT]; GRID_WIDTH])
    }
}

#[allow(clippy::cast_possible_truncation)]
impl Grid<Option<char>> {
    /// Scans a `Grid` for words.
    pub fn scan_for_words(&self) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        let mut current_word: String = String::new();

        // Scan horizontally.
        for y in 0..GRID_WIDTH {
            for x in 0..GRID_HEIGHT {
                if let Some(letter) = self[GridIndex(x as u8, y as u8)] {
                    let direction: Direction =
                        self.letter_adjacent(GridIndex(x as u8, y as u8).into());
                    if direction != Direction::Vertical {
                        current_word.push(letter);
                    }
                } else if !current_word.is_empty() {
                    output.push(current_word);
                    current_word = String::new();
                }
            }
        }

        if !current_word.is_empty() {
            output.push(current_word);
            current_word = String::new();
        }

        // Scan vertically.
        for x in 0..GRID_HEIGHT {
            for y in 0..GRID_WIDTH {
                if let Some(letter) = self[GridIndex(x as u8, y as u8)] {
                    let direction = self.letter_adjacent(GridIndex(x as u8, y as u8).into());
                    if direction != Direction::Horizontal {
                        current_word.push(letter);
                    }
                } else if current_word.chars().count() == 1 {
                    current_word = String::new();
                } else if !current_word.is_empty() {
                    output.push(current_word);
                    current_word = String::new();
                }
            }
        }

        if !current_word.is_empty() {
            output.push(current_word);
        }

        output
    }

    /// Checks if there's a letter adjacent to this coordinate.
    /// Returns the direction the letter was found in.
    fn letter_adjacent(&self, coordinate: Coordinate) -> Direction {
        let horizontal: bool =
            if let (x, false) = coordinate.overflowing_add(Coordinate(1, 0)) {
                self[x].is_some()
            } else {
                false
            } || if let (x, false) = coordinate.overflowing_add(Coordinate(-1, 0)) {
                self[x].is_some()
            } else {
                false
            };
        let vertical: bool = if let (x, false) = coordinate.overflowing_add(Coordinate(0, 1)) {
            self[x].is_some()
        } else {
            false
        } || if let (x, false) = coordinate.overflowing_add(Coordinate(0, -1))
        {
            self[x].is_some()
        } else {
            false
        };

        if horizontal && vertical {
            return Direction::Both;
        } else if horizontal {
            return Direction::Horizontal;
        } else if vertical {
            return Direction::Vertical;
        }
        Direction::None
    }

    /// Checks every word to ensure it is in the dictionary.
    pub fn validate_words(words: &[String], dictionary: &HashSet<String>) -> Result<(), Error> {
        for word in words.iter().cloned() {
            if !dictionary.contains(&word) {
                return Err(Error::InvalidWord(word));
            }
        }
        Ok(())
    }

    /// Depth-first search to traverse all connected cells.
    fn dfs(&self, visited: &mut Grid<bool>, coordinate: Coordinate) {
        /// Every direction that DFS can go in.
        const DIRECTIONS: [Coordinate; 4] = [
            Coordinate(1, 0),
            Coordinate(-1, 0),
            Coordinate(0, 1),
            Coordinate(0, -1),
        ];

        visited[coordinate] = true;
        for direction in DIRECTIONS {
            let (new_coord, overflowed) = coordinate.overflowing_add(direction);
            if overflowed {
                continue;
            }
            if self[new_coord].is_some() && !visited[new_coord] {
                self.dfs(visited, new_coord);
            }
        }
    }

    /// Ensures that all words are connected.
    pub fn validate_connectivity(&self) -> Result<(), Error> {
        let mut visited: Grid<bool> = Grid::default();

        // Find the first occupied cell to start DFS.
        let mut start: Option<GridIndex> = None;
        for x in 0..GRID_HEIGHT {
            for y in 0..GRID_WIDTH {
                if self[GridIndex(x as u8, y as u8)].is_some() {
                    start = Some(GridIndex(x as u8, y as u8));
                    break;
                }
            }
            if start.is_some() {
                break;
            }
        }

        // Perform DFS from the first occupied cell.
        if let Some(index) = start {
            self.dfs(&mut visited, index.into());

            // Check if all occupied cells are visited.
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    if self[GridIndex(x as u8, y as u8)].is_some()
                        && !visited[GridIndex(x as u8, y as u8)]
                    {
                        // Found an unconnected cell!
                        return Err(Error::WordsNotConnected);
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn score_grid(words: &[String], scoretable: &HashMap<char, i64>) -> i64 {
        /*
            Stale (previously used) words: 0.8x
            Length of word: 1-3 is 1x, 4-6 is 1.5x, 7-9 is 2x, 10+ is 2.5x
        */

        let mut seen = HashSet::new();
        let stale = words
            .iter()
            .filter(|s| !seen.insert(*s)) // Keep only the first instance of each string
            .collect::<Vec<_>>();
        let mut change: i64 = 0;

        for word in words {
            let mut word_score: f64 = 0.0;

            // Score letters
            for tile in word.chars() {
                word_score += *scoretable.get(&tile).unwrap_or(&0) as f64;
            }

            // Length multiplier
            word_score *= match word.len() {
                1..=3 => 1.0,
                4..=6 => 1.5,
                7..=9 => 2.0,
                _ => 2.5,
            };

            // Stale word check
            // rescoring every word is a feature, not a bug. trust me. - clover <3
            for _ in 0..stale.iter().filter(|x: &&&String| **x == word).count() {
                word_score *= 0.8;
            }

            change += word_score as i64;
        }

        change
    }
}

impl<T> Default for Grid<T>
where T: Default + Copy {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    None,
    Vertical,
    Horizontal,
    Both,
}
