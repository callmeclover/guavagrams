mod grid;
mod util;

use std::collections::HashSet;

use color_eyre::Result;
use grid::{Coordinate, Grid};

fn main() -> Result<()> {
    let mut playfield: Grid = Grid::new();

    playfield[Coordinate(0, 0)] = Some('h');
    playfield[Coordinate(1, 0)] = Some('e');
    playfield[Coordinate(2, 0)] = Some('l');
    playfield[Coordinate(3, 0)] = Some('l');
    playfield[Coordinate(4, 0)] = Some('o');
    playfield[Coordinate(0, -1)] = Some('i');
    //playfield[Coordinate(6, 0)] = Some('a');
    //playfield[Coordinate(6, -1)] = Some('n');
    //playfield[Coordinate(6, -2)] = Some('d');

    let words_on_field: Vec<String> = playfield.scan_for_words();
    println!("{words_on_field:?}");

    playfield.validate_connectivity()?;

    Grid::validate_words(
        &words_on_field,
        &HashSet::from([
            "hello".to_string(),
            "hi".to_string(),
            "john".to_string(),
            "and".to_string(),
            "a".to_string(),
        ]),
    )?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not all words are connected!")]
    WordsNotConnected,
    #[error("Invalid word \"{0}\"!")]
    InvalidWord(String),
}
