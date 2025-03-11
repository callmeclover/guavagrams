#![feature(mixed_integer_ops_unsigned_sub)]

mod camera;
mod dictionary;
mod grid;
mod util;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use camera::Camera;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use dictionary::{get_dictionary, list_dictionaries, Distribution};
use grid::{Coordinate, Grid, SharedGrid};
use itertools::Itertools;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use ratatui::{
    layout::Rect,
    prelude::*,
    style::Styled,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use util::{format_duration, format_tile_list};

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
    };

    let mut status: Span = "".set_style(Style::new().fg(Color::Black).bg(Color::White));

    let mut terminal = ratatui::init();
    loop {
        terminal
            .draw(|frame: &mut Frame| {
                let layout: Rc<[Rect]> = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .spacing(1)
                    .split(frame.area());

                let block: Block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .title(format!(
                        " Game, ({}) ",
                        format_duration(
                            state
                                .game_end
                                .unwrap_or_else(Instant::now)
                                .duration_since(state.game_start)
                        )
                    ))
                    .title_bottom(status.clone())
                    .title_alignment(Alignment::Center);
                let block_layout: Rc<[Rect]> = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .spacing(1)
                    .split(block.inner(layout[0]));

                let tiles_block: Block = Block::default()
                    .border_type(BorderType::Plain)
                    .borders(Borders::TOP)
                    .title("Your Tiles")
                    .title_alignment(Alignment::Center);

                frame.render_widget(&block, layout[0]);

                let keys = [
                    ("↑/↓/←/→", "Move"),
                    ("Any Letter", "Place"),
                    ("Del", "Pick Up"),
                    ("Ctrl + Any Letter", "Trade In"),
                    ("Shift + G", "Peel/Guavagrams!"),
                    ("Shift + Q/Esc", "Quit"),
                ];
                let mut lines = vec![
                    Line::raw(format!("Coordinates: {}", state.camera.cursor)),
                    Line::raw(format!("Tiles left in pile: {}", state.tileset.0.len())),
                    Line::raw(format!("Score: {}", state.score)),
                    Line::default(),
                ];

                lines.append(
                    &mut keys
                        .iter()
                        .map(|(key, desc)| {
                            let key: Span = Span::styled(format!(" {key} "), Style::new().cyan());
                            let desc: Span = Span::styled(format!(" {desc} "), Style::default());
                            Line::from(vec![key, desc])
                        })
                        .collect_vec(),
                );

                frame.render_widget(Paragraph::new(lines), block_layout[0]);
                frame.render_widget(&tiles_block, block_layout[1]);
                frame.render_widget(
                    Paragraph::new(Text::from(format_tile_list(&state.tileset.1)))
                        .wrap(Wrap { trim: false }),
                    tiles_block.inner(block_layout[1]),
                );
                frame.render_widget(&mut state.camera, layout[1]);
            })
            .expect("failed to draw frame");

        if event::poll(Duration::from_millis(50))? {
            match event_handler(&mut state) {
                Ok(response) => match response {
                    EventResponse::Quit => break,
                    EventResponse::ChangeStatus(new_status) => status = new_status,
                    EventResponse::Pass => (),
                },
                Err(exception) => {
                    status = exception.to_string().set_style(Style::new().fg(Color::Red));
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

/// The event logic.
fn event_handler(state: &mut GameState) -> Result<EventResponse, Error> {
    if let Event::Key(event) = event::read().expect("failed to read event") {
        if event.kind == KeyEventKind::Press {
            match event.code {
                // Quit game
                KeyCode::Esc | KeyCode::Char('Q') => return Ok(EventResponse::Quit),

                // Movement controls
                KeyCode::Right => state.camera += Coordinate(1, 0),
                KeyCode::Left => state.camera += Coordinate(-1, 0),
                KeyCode::Up => state.camera += Coordinate(0, 1),
                KeyCode::Down => state.camera += Coordinate(0, -1),

                // Letter controls
                KeyCode::Char('G') => {
                    if !state.tileset.1.is_empty() {
                        return Err(Error::HandHasTiles);
                    }

                    let handle = state.camera.grid.lock().unwrap();
                    if let Err(exception) = 
                    handle.validate_connectivity().and_then(|_| Grid::validate_words(&handle.scan_for_words(), &state.dictionary))
                    {
                        state.score -= 5;
                        return Err(exception);
                    }

                    drop(handle);

                    if state.tileset.0.is_empty() {
                        state.game_end = Some(Instant::now());
                        return Ok(EventResponse::ChangeStatus(
                            "Guavagrams!".set_style(Style::new().fg(Color::Green)),
                        ));
                    }
                    state
                        .tileset
                        .1
                        .append(&mut Distribution::pull_from_pile(&mut state.tileset.0, 1)?);
                    return Ok(EventResponse::ChangeStatus(
                        "Peel!".set_style(Style::new().fg(Color::Green)),
                    ));
                }
                KeyCode::Char(letter)
                    if event.modifiers.contains(KeyModifiers::CONTROL)
                        && state.tileset.1.contains(&letter)
                        && state.game_end.is_none() =>
                {
                    if state.tileset.0.len() >= 3 {
                        state
                            .tileset
                            .1
                            .append(&mut Distribution::pull_from_pile(&mut state.tileset.0, 3)?);
                        state.tileset.0.push(
                            state.tileset.1.remove(
                                state
                                    .tileset
                                    .1
                                    .iter()
                                    .position(|x: &char| *x == letter)
                                    .ok_or(Error::NoMoreTiles)?,
                            ),
                        );
                        state.score -= 5;
                        state.tileset.0.shuffle(&mut ThreadRng::default());
                    }
                }
                KeyCode::Char(letter)
                    if (letter.is_lowercase() || !letter.is_alphabetic())
                        && state.distribution.contains_letter(letter)
                        && state.tileset.1.contains(&letter)
                        && state.game_end.is_none() =>
                {
                    // Check if a tile was actually put down before removing it from our hand.
                    if state.camera.put(letter) {
                        state.tileset.1.remove(
                            state
                                .tileset
                                .1
                                .iter()
                                .position(|x: &char| *x == letter)
                                .unwrap(),
                        );
                        state.score += state.scoretable[&letter];
                    }
                }
                KeyCode::Backspace if state.game_end.is_none() => {
                    if let Some(tile) = state.camera.pick_up() {
                        state.tileset.1.push(tile);
                        state.score -= state.scoretable[&tile];
                    }
                }
                _ => (),
            }
        }
    }

    Ok(EventResponse::Pass)
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
