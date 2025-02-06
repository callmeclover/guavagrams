use std::{
    collections::HashSet, path::{Path, PathBuf}
};

use csv::{Reader, StringRecord};
use rand::{distr::Distribution as _, rngs::ThreadRng};
use walkdir::{DirEntry, WalkDir};

use crate::util::create_weights;

/// Recursively lists every file in `./dictionaries/`.
pub fn list_dictionaries() -> Vec<PathBuf> {
    WalkDir::new("dictionaries")
        .into_iter()
        .filter_map(|entry: Result<DirEntry, walkdir::Error>| {
            entry.ok().and_then(|x| {
                if x.file_type().is_file() {
                    return Some(x);
                };
                None
            })
        })
        .map(|entry: DirEntry| entry.path().to_path_buf())
        .collect()
}

pub fn get_dictionary(path: &Path) -> csv::Result<HashSet<String>> {
    Ok(Reader::from_path(path)?
        .into_records()
        .map(|x: Result<StringRecord, csv::Error>| {
            x.expect("Loading dictionary failed.")
                .as_slice()
                .to_string()
        })
        .collect())
}

pub type LetterDistribution = Vec<(char, usize)>;

#[derive(Debug)]
pub enum Distribution {
    /// The distributed rarity of tiles from a dictionary.
    Dictionary(LetterDistribution),
    Bananagrams,
    Scrabble,
}

impl Distribution {
    /*const SCRABBLE: LetterDistribution = vec![
        ('a'),
        ('b'),
        ('c'),
        ('d'),
        ('e'),
        ('f'),
        ('g'),
        ('h'),
        ('i'),
        ('j'),
        ('k'),
        ('l'),
        ('m'),
        ('n'),
        ('o'),
        ('p'),
        ('q'),
        ('r'),
        ('s'),
        ('t'),
        ('u'),
        ('v'),
        ('w'),
        ('x'),
        ('y'),
        ('z'),
    ];
    const BANANAGRAMS: LetterDistribution = vec![
        ('a', 13),
    ('b', 3),
    ('c', 3),
    ('d', 6),
    ('e', 18),
    ('f', 3),
    ('g', 4),
    ('h', 3),
    ('i', 12),
    ('j', 2),
    ('k', 2),
    ('l'),
    ('m'),
    ('n'),
    ('o'),
    ('p'),
    ('q'),
    ('r'),
    ('s'),
    ('t'),
    ('u'),
    ('v'),
    ('w'),
    ('x'),
    ('y'),
    ('z'),];*/

    /// Creates a `Distribution::Dictionary` from a `HashSet`.
    pub fn from_dictionary(dictionary: &HashSet<String>) -> Self {
        let mut characters: Vec<char> = Vec::new();
        for word in dictionary {
            characters.append(
                &mut word
                    .chars()
                    .filter(|character: &char| !character.is_whitespace())
                    .collect(),
            );
        }
        // Sort so that chunking works
        characters.sort_unstable();
        Self::Dictionary(
            characters
                // Chunk all characters into seperate, smaller arrays
                .chunk_by(|x: &char, y: &char| x == y)
                // Map each `x` to `(x, count)`
                .map(|x: &[char]| (x[0], x.len()))
                .collect(),
        )
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn create_pile_internals(letter_distribution: &LetterDistribution, amount: usize) -> Vec<char> {
        let mut output: Vec<char> = Vec::new();
        let total_letters: f64 = letter_distribution.iter().fold(0.0, |total: f64, (.., curr)| total + *curr as f64);
        for (tile, frequency) in letter_distribution.clone() {
            output.extend_from_slice(&vec![
                tile;
                (frequency as f64 / (total_letters / amount as f64)).round()
                    as usize
            ]);
        }
        output
    }

    pub fn create_pile(&self, amount: usize) -> Vec<char> {
        match self {
            Self::Dictionary(letter_distribution) => Self::create_pile_internals(letter_distribution, amount),
            Self::Bananagrams => todo!(),
            Self::Scrabble => todo!(),
        }
    }

    pub fn pull_from_pile(pile: &mut Vec<char>) -> char {
        todo!()
    }

    pub fn pull_endless(&self) -> char {
        let mut rng: ThreadRng = ThreadRng::default();
        match self {
            Self::Dictionary(letter_distribution) => {
                letter_distribution[create_weights(letter_distribution).sample(&mut rng)].0
            }
            Self::Bananagrams => todo!(),
            Self::Scrabble => todo!(),
        }
    }
}
