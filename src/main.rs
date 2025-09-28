mod camera;
mod dictionary;
mod grid;
mod ui;
mod util;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use camera::Camera;
use color_eyre::Result;
use crossterm::event;
use dictionary::{Distribution, get_dictionary, list_dictionaries};
use grid::{Grid, SharedGrid};
use ratatui::{prelude::*, style::Styled};
use ui::{draw, event_handler};

#[derive(Clone)]
struct GameState {
    dictionary: HashSet<String>,
    camera: Camera,
    distribution: Distribution,
    tileset: (Vec<char>, Vec<char>),
    game_start: Instant,
    game_end: Option<Instant>,
    score: i64,
    scoretable: HashMap<char, i64>,
    status: Span<'static>,
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn main() -> Result<()> {
    color_eyre::install()?;

    let dictionary_list: Vec<PathBuf> = list_dictionaries();
    let dictionary: HashSet<String> = get_dictionary(&dictionary_list[0])?;

    let grid: SharedGrid = Arc::new(Mutex::new(Grid::new()));
    let mut state: GameState = GameState {
        dictionary,
        camera: Camera::new(grid),
        distribution: Distribution::Bananagrams,
        tileset: {
            let mut pile: Vec<char> = Distribution::Bananagrams.create_pile(144);
            let mut hand: Vec<char> = Distribution::pull_from_pile(&mut pile, 21).unwrap();
            hand.sort_unstable();
            (pile, hand)
        },
        game_start: Instant::now(),
        game_end: None,
        score: 0,
        scoretable: HashMap::from([
            ('a', 1),
            ('b', 3),
            ('c', 3),
            ('d', 2),
            ('e', 1),
            ('f', 4),
            ('g', 2),
            ('h', 4),
            ('i', 1),
            ('j', 8),
            ('k', 5),
            ('l', 1),
            ('m', 3),
            ('n', 1),
            ('o', 1),
            ('p', 3),
            ('q', 10),
            ('r', 1),
            ('s', 1),
            ('t', 1),
            ('u', 1),
            ('v', 4),
            ('w', 4),
            ('x', 8),
            ('y', 4),
            ('z', 10),
        ]),
        status: "".set_style(Style::new().fg(Color::Black).bg(Color::White)),
    };

    let mut terminal = ratatui::init();
    loop {
        terminal
            .draw(|frame| draw(frame, &mut state))
            .expect("failed to draw frame");

        if event::poll(Duration::from_millis(50))? {
            match event_handler(&mut state) {
                Ok(response) => match response {
                    EventResponse::Quit => break,
                    EventResponse::ChangeStatus(new_status) => state.status = new_status,
                    EventResponse::Pass => (),
                },
                Err(exception) => {
                    state.status = exception.to_string().set_style(Style::new().fg(Color::Red));
                }
            }
        }
    }
    ratatui::restore();

    Ok(())
}

#[derive(PartialEq, Eq, Clone)]
enum EventResponse {
    Pass,
    ChangeStatus(Span<'static>),
    Quit,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("Not all words are connected!")]
    WordsNotConnected,
    #[error("Invalid word \"{0}\"!")]
    InvalidWord(String),
    #[error("The pile's all out of tiles, or there isn't enough to pull!")]
    NoMoreTiles,
    #[error("You still have tiles in your hand!")]
    HandHasTiles,
}
