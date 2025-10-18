use std::ops::AddAssign;

use ratatui::{
    layout::Rect,
    style::{Color, Style, Styled},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::grid::{Coordinate, Grid, GridIndex};

#[derive(Clone)]
pub struct Camera {
    pub grid: Grid<Option<char>>,
    pub cursor: Coordinate,
    current_screen_space: Rect,
}

impl Camera {
    pub fn new(grid: Grid<Option<char>>) -> Self {
        Self {
            grid,
            cursor: Coordinate::default(),
            current_screen_space: Rect::default(),
        }
    }

    pub fn put(&mut self, letter: char) -> bool {
        if self.grid[self.cursor].is_some() {
            return false;
        }
        self.grid[self.cursor].get_or_insert(letter);
        true
    }

    pub fn pick_up(&mut self) -> Option<char> {
        let tile: Option<char> = self.grid[self.cursor];
        self.grid[self.cursor] = None;
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
                u8::MIN.saturating_add(area.height as u8 / 2),
                u8::MAX.saturating_sub(area.height as u8 / 2),
            );
            let clamped_x = cursor_index.0.clamp(
                u8::MIN.saturating_add(area.width as u8 / 4),
                u8::MAX.saturating_sub(area.width as u8 / 4),
            );

            for y in (clamped_y - (area.height / 2) as u8)..=(clamped_y + (area.height / 2) as u8) {
                let mut line: Line = Line::default();
                for x in clamped_x.saturating_sub((area.width / 4) as u8)
                    ..=clamped_x.saturating_add((area.width / 4) as u8)
                {
                    let span = if GridIndex(x, y) == cursor_index {
                        self.grid[GridIndex(x, y)]
                            .unwrap_or('.')
                            .to_string()
                            .set_style(Style::new().fg(Color::Black).bg(Color::White))
                    } else {
                        self.grid[GridIndex(x, y)]
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
