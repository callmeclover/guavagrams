use std::ops::AddAssign;

use crossterm::style::Stylize;
use ratatui::{
    style::{Color, Style, Styled},
    text::{Line, Span, ToSpan},
    widgets::{Paragraph, Widget},
};

use crate::grid::{Coordinate, GridIndex, SharedGrid};

pub struct Camera {
    grid: SharedGrid,
    pub cursor: Coordinate,
}

impl Camera {
    pub fn new(grid: SharedGrid) -> Self {
        Self {
            grid: grid,
            cursor: Coordinate::default(),
        }
    }

    pub fn put(&self, letter: char) {
        self.grid.lock().unwrap()[self.cursor] = Some(letter);
    }
}

impl AddAssign<Coordinate> for Camera {
    fn add_assign(&mut self, rhs: Coordinate) {
        self.cursor += rhs;
    }
}

impl Widget for &Camera {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let text: Vec<Line> = {
            let mut output: Vec<Line> = Vec::new();
            for y in (<Coordinate as Into<GridIndex>>::into(self.cursor).1
                - (area.height / 2) as u8)
                ..=(<Coordinate as Into<GridIndex>>::into(self.cursor).1 + (area.height / 2) as u8)
            {
                let mut line: Line = Line::default();
                for x in (<Coordinate as Into<GridIndex>>::into(self.cursor).0
                    - (area.width / 2) as u8)
                    ..=(<Coordinate as Into<GridIndex>>::into(self.cursor).0
                        + (area.width / 2) as u8)
                {
                    let span = if GridIndex(x, y) == self.cursor.into() {
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
                }
                output.push(line);
            }
            output
        };

        Paragraph::new(text).render(area, buf);
    }
}
