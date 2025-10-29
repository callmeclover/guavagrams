mod camera;
mod dictionary;
mod grid;
mod ui;
mod util;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use camera::Camera;
use color_eyre::Result;
#[cfg(not(target_family = "wasm"))]
use crossterm::event;
use dictionary::{Distribution, get_dictionary};
use grid::Grid;
use ratatui::prelude::*;
use ratzilla::{WebGl2Backend, WebRenderer, backend::webgl2::WebGl2BackendOptions};
use ui::{draw, event_handler};
use web_time::Instant;

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
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let grid: Grid<Option<char>> = Grid::default();
    let state: Rc<RefCell<GameState>> = Rc::new(RefCell::new(GameState {
        dictionary: get_dictionary()?,
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
        status: Span::default(),
    }));

    let terminal = Terminal::new(
        WebGl2Backend::new_with_options(WebGl2BackendOptions::new().enable_mouse_selection())
            .expect("could not build webgl2 backend"),
    )?;

    terminal.on_key_event({
        let state = state.clone();
        move |event| {
            let mut state = state.borrow_mut();
            match event_handler(&event, &mut state) {
                Ok(response) => match response {
                    EventResponse::ChangeStatus(new_status) => state.status = new_status,
                    EventResponse::Quit | EventResponse::Pass => (),
                },
                Err(exception) => {
                    state.status = exception.to_string().red();
                }
            }
        }
    });
    terminal.draw_web(move |frame| {
        let mut state = state.borrow_mut();
        draw(frame, &mut state);
    });

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
