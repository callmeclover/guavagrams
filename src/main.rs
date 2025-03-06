mod camera;
mod dictionary;
mod grid;
mod util;

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use camera::Camera;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use dictionary::{get_dictionary, list_dictionaries, Distribution};
use grid::{Coordinate, Grid, SharedGrid};
use ratatui::{layout::Rect, text::Line, Frame};

fn main() -> Result<()> {
    color_eyre::install()?;

    let dictionary_list: Vec<PathBuf> = list_dictionaries();
    let dictionary: HashSet<String> = get_dictionary(&dictionary_list[0])?;
    let distribution: Distribution = Distribution::from_dictionary(&dictionary);
    let grid: SharedGrid = Arc::new(Mutex::new(Grid::new()));
    let mut camera: Camera = Camera::new(grid);

    let mut hand: Vec<char> = Vec::new();

    let mut terminal = ratatui::init();
    loop {
        terminal
            .draw(|frame: &mut Frame| {
                frame.render_widget(&camera, Rect::new(0, 0, 35, 15));
            })
            .expect("failed to draw frame");
        if let Event::Key(event) = event::read().expect("failed to read event") {
            if event.kind == KeyEventKind::Press {
                match event.code {
                    KeyCode::Esc => break,
                    KeyCode::Right => camera += Coordinate(1, 0),
                    KeyCode::Left => camera += Coordinate(-1, 0),
                    KeyCode::Up => camera += Coordinate(0, 1),
                    KeyCode::Down => camera += Coordinate(0, -1),
                    KeyCode::Char(letter) if distribution.contains_letter(letter) => camera.put(letter),
                    KeyCode::Backspace => {
                        if let Some(tile) = camera.pick_up() {
                            hand.push(tile);
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    ratatui::restore();

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not all words are connected!")]
    WordsNotConnected,
    #[error("Invalid word \"{0}\"!")]
    InvalidWord(String),
    #[error("You're all out of tiles, or you don't have enough to pull!")]
    NoMoreTiles,
}
