mod dictionary;
mod grid;
mod util;

use std::{collections::HashSet, path::PathBuf};

use color_eyre::Result;
use dictionary::{get_dictionary, list_dictionaries, Distribution};

fn main() -> Result<()> {
    let dictionary_list: Vec<PathBuf> = list_dictionaries();
    let dictionary: HashSet<String> = get_dictionary(&dictionary_list[0])?;

    let distribution: Distribution = Distribution::from_dictionary(&dictionary);
    println!("{distribution:?}");

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not all words are connected!")]
    WordsNotConnected,
    #[error("Invalid word \"{0}\"!")]
    InvalidWord(String),
}
