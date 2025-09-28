use std::{rc::Rc, time::Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use itertools::Itertools;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled, Stylize as _},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::{
    Error, EventResponse, GameState,
    dictionary::Distribution,
    grid::{Coordinate, Grid},
    util::{format_duration, format_tile_list},
};

pub fn draw(frame: &mut Frame, state: &mut GameState) {
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
        .title_bottom(state.status.clone())
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
        Paragraph::new(Text::from(format_tile_list(&state.tileset.1))).wrap(Wrap { trim: false }),
        tiles_block.inner(block_layout[1]),
    );
    frame.render_widget(&mut state.camera, layout[1]);
}

/// The event logic.
#[allow(clippy::cast_precision_loss)]
pub fn event_handler(state: &mut GameState) -> Result<EventResponse, Error> {
    if let Event::Key(event) = event::read().expect("failed to read event")
        && event.kind == KeyEventKind::Press
    {
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
                let words: Vec<String> = handle.scan_for_words();
                if let Err(exception) = handle
                    .validate_connectivity()
                    .and_then(|()| Grid::validate_words(&words, &state.dictionary))
                {
                    state.score -= state.score / 20;
                    return Err(exception);
                }
                drop(handle);

                state.score += Grid::score_grid(&words, &state.scoretable);

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

                    state.score -= state.score / 20;
                    state.tileset.0.shuffle(&mut ThreadRng::default());
                }

                return Ok(EventResponse::ChangeStatus(
                    "Deducted 5% of points for trading in tiles."
                        .set_style(Style::new().fg(Color::Red)),
                ));
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
                }
            }
            KeyCode::Backspace if state.game_end.is_none() => {
                if let Some(tile) = state.camera.pick_up() {
                    state.tileset.1.push(tile);
                }
            }
            _ => (),
        }
    }

    Ok(EventResponse::Pass)
}
