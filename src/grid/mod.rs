pub mod index;

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub use index::{Coordinate, GridIndex};

use crate::{box_array, Error};

/// The amount of columns in a grid.
/// The default is 256.
const GRID_WIDTH: usize = u8::MAX as usize + 1;
/// The amount of rows in the grid.
/// The default is 256.
const GRID_HEIGHT: usize = u8::MAX as usize + 1;

/// A 2D, fixed size array on the heap.
#[derive(Debug)]
pub struct Grid(Box<[[Option<char>; GRID_HEIGHT]; GRID_WIDTH]>);

#[allow(clippy::cast_possible_truncation)]
impl Grid {
    /// Constructs a `Grid`.
    pub fn new() -> Self {
        Self(box_array![[None; GRID_HEIGHT]; GRID_WIDTH])
    }

    /// Scans a `Grid` for words.
    pub fn scan_for_words(&self) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        let mut current_word: String = String::new();

        // Scan horizontally.
        for y in 0..GRID_WIDTH {
            for x in 0..GRID_HEIGHT {
                if let Some(letter) = self[GridIndex(x as u8, y as u8)] {
                    let direction: Direction = self.letter_adjacent(GridIndex(x as u8, y as u8).into());
                    if !(direction == Direction::Vertical) {
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
                    if !(self.letter_adjacent(GridIndex(x as u8, y as u8).into()) == Direction::Horizontal) {
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
        let horizontal: bool = self[coordinate + Coordinate(1, 0)].is_some()
            || self[coordinate - Coordinate(1, 0)].is_some();
        let vertical: bool = self[coordinate + Coordinate(0, 1)].is_some()
            || self[coordinate - Coordinate(0, 1)].is_some();

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
    fn dfs(&self, visited: &mut BoolGrid, coordinate: Coordinate) {
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
        let mut visited: BoolGrid = BoolGrid::new();

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
            for x in 0..GRID_HEIGHT {
                for y in 0..GRID_WIDTH {
                    if self[GridIndex(x as u8, y as u8)].is_some() && !visited[GridIndex(x as u8, y as u8)] {
                        // Found an unconnected cell!
                        return Err(Error::WordsNotConnected);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

/// A 2D, fixed size array of booleans on the heap.
#[derive(Debug)]
pub struct BoolGrid(Box<[[bool; GRID_HEIGHT]; GRID_WIDTH]>);

impl BoolGrid {
    /// Constructs a `BoolGrid`.
    pub fn new() -> Self {
        Self(box_array![[false; GRID_HEIGHT]; GRID_WIDTH])
    }
}

pub type SharedGrid = Arc<Mutex<Grid>>;

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    None,
    Vertical,
    Horizontal,
    Both,
}
