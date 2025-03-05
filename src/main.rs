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
use crossterm::event::{self, Event, KeyCode};
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

    let mut terminal = ratatui::init();
    loop {
        terminal
            .draw(|frame: &mut Frame| {
                frame.render_widget(&camera, Rect::new(0, 0, 35, 15));
                frame.render_widget(Line::raw(format!("{}", camera.cursor)), frame.area());
            })
            .expect("failed to draw frame");
        match event::read().expect("failed to read event") {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Right => camera += Coordinate(1, 0),
                KeyCode::Left => camera += Coordinate(-1, 0),
                KeyCode::Up => camera += Coordinate(0, 1),
                KeyCode::Down => camera += Coordinate(0, -1),
                KeyCode::Char(' ') => camera.put('a'),
                _ => (),
            },
            _ => (),
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
