mod camera;
mod dictionary;
mod grid;
mod util;

use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use camera::Camera;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use dictionary::Distribution;
use grid::{Coordinate, Grid, SharedGrid};
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, Borders},
    Frame,
};
use util::format_duration;

fn main() -> Result<()> {
    color_eyre::install()?;

    let distribution: Distribution = Distribution::Bananagrams;
    let grid: SharedGrid = Arc::new(Mutex::new(Grid::new()));
    let mut camera: Camera = Camera::new(grid);

    let mut hand: Vec<char> = Vec::new();
    let pile: Vec<char> = distribution.create_pile(144);

    let mut terminal = ratatui::init();
    let game_start = Instant::now();

    loop {
        terminal
            .draw(|frame: &mut Frame| {
                let layout: Rc<[Rect]> = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .spacing(1)
                    .split(Rect::new(0, 0, frame.area().width / 2, frame.area().height));

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .title(format!(
                        " Game, ({}) ",
                        format_duration(game_start.elapsed())
                    ))
                    .title_alignment(Alignment::Center);
                frame.render_widget(&block, layout[0]);
                //frame.render_widget(Line::raw("test"), block.inner(layout[0])); TODO: Make show game stats
                frame.render_widget(&camera, layout[1]);
            })
            .expect("failed to draw frame");
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(event) = event::read().expect("failed to read event") {
                if event.kind == KeyEventKind::Press {
                    match event.code {
                        KeyCode::Esc => break,
                        KeyCode::Right => camera += Coordinate(1, 0),
                        KeyCode::Left => camera += Coordinate(-1, 0),
                        KeyCode::Up => camera += Coordinate(0, 1),
                        KeyCode::Down => camera += Coordinate(0, -1),
                        KeyCode::Char(letter)
                            if distribution.contains_letter(letter) && hand.contains(&letter) =>
                        {
                            camera.put(letter);
                        }
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
