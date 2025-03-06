use std::{
    ops::{AddAssign, Index},
    u8,
};

use ratatui::{
    layout::Rect,
    style::{Color, Style, Styled},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::grid::{Coordinate, GridIndex, SharedGrid};

#[derive(Clone)]
pub struct Camera {
    pub grid: SharedGrid,
    pub cursor: Coordinate,
    current_screen_space: Rect,
}

impl Camera {
    pub fn new(grid: SharedGrid) -> Self {
        Self {
            grid,
            cursor: Coordinate::default(),
            current_screen_space: Rect::default(),
        }
    }

    pub fn put(&self, letter: char) -> bool {
        let mut handle = self.grid.lock().unwrap();
        if handle[self.cursor].is_some() {
            return false;
        }
        handle[self.cursor].get_or_insert(letter);
        true
    }

    pub fn pick_up(&self) -> Option<char> {
        let tile: Option<char> = self.grid.lock().unwrap()[self.cursor];
        self.grid.lock().unwrap()[self.cursor] = None;
        tile
    }
}

impl AddAssign<Coordinate> for Camera {
    fn add_assign(&mut self, rhs: Coordinate) {
        self.cursor += rhs;
    }
}
#[allow(clippy::cast_possible_truncation)]
impl Widget for &mut Camera {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.current_screen_space = area;
        let text: Vec<Line> = {
            let mut output: Vec<Line> = Vec::new();
            let cursor_index: GridIndex = self.cursor.into();
            let clamped_y = cursor_index.1.clamp(
                u8::MIN + area.height as u8 / 2,
                u8::MAX - area.height as u8 / 2,
            );
            let clamped_x = cursor_index.0.clamp(
                u8::MIN + area.width as u8 / 4,
                u8::MAX - area.width as u8 / 4,
            );

            for y in (clamped_y - (area.height / 2) as u8)..(clamped_y + (area.height / 2) as u8) {
                let mut line: Line = Line::default();
                for x in (clamped_x - (area.width / 4) as u8)..(clamped_x + (area.width / 4) as u8)
                {
                    let span = if GridIndex(x, y) == cursor_index {
                        self.grid.lock().unwrap()[GridIndex(x, y)]
                            .unwrap_or('.')
                            .to_string()
                            .set_style(Style::new().fg(Color::Black).bg(Color::White))
                    } else {
                        self.grid.lock().unwrap()[GridIndex(x, y)]
                            .unwrap_or('.')
                            .to_string()
                            .set_style(Style::default())
                    };
                    line.push_span(span);
                    line.push_span(" ");
                }
                output.push(line);
            }
            output
        };

        Paragraph::new(text).render(area, buf);
    }
}
