use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use csv::{Reader, StringRecord};
use rand::{distr::Distribution as _, rngs::ThreadRng};
use walkdir::{DirEntry, WalkDir};

use crate::{util::create_weights, Error};

/// Recursively lists every file in `./dictionaries/`.
pub fn list_dictionaries() -> Vec<PathBuf> {
    WalkDir::new("dictionaries")
        .into_iter()
        .filter_map(|entry: Result<DirEntry, walkdir::Error>| {
            entry.ok().and_then(|x: DirEntry| {
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
    ];*/
    const BANANAGRAMS: LazyLock<LetterDistribution> = LazyLock::new(|| {
        vec![
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
            ('l', 5),
            ('m', 3),
            ('n', 8),
            ('o', 11),
            ('p', 3),
            ('q', 2),
            ('r', 9),
            ('s', 6),
            ('t', 9),
            ('u', 6),
            ('v', 3),
            ('w', 3),
            ('x', 2),
            ('y', 3),
            ('z', 2),
        ]
    });

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

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn create_pile_internals(letter_distribution: &LetterDistribution, amount: usize) -> Vec<char> {
        let mut output: Vec<char> = Vec::new();
        let total_letters: f64 = letter_distribution
            .iter()
            .fold(0.0, |total: f64, (.., curr)| total + *curr as f64);
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
            Self::Dictionary(letter_distribution) => {
                Self::create_pile_internals(letter_distribution, amount)
            }
            Self::Bananagrams => Self::create_pile_internals(&Self::BANANAGRAMS, amount),
            Self::Scrabble => todo!(),
        }
    }

    pub fn pull_from_pile(pile: &mut [char], amount: usize) -> Result<Vec<&char>, Error> {
        if pile.len() < amount {
            return Err(Error::NoMoreTiles);
        }
        Ok(pile.iter().take(amount).collect())
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
