use std::ops::AddAssign;

use ratatui::{
    layout::Rect,
    style::Stylize as _,
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::grid::{Coordinate, GRID_SIZE, Grid, GridIndex};

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

impl Widget for &mut Camera {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let ubound_x = GRID_SIZE.bound_x.as_unsigned();
        let ubound_y = GRID_SIZE.bound_y.as_unsigned();
        self.current_screen_space = area;
        let text: Vec<Line> = {
            let mut output: Vec<Line> = Vec::new();
            let cursor_index: GridIndex = self.cursor.into();

            let clamped_x = cursor_index.0.clamp(
                ubound_x.0.saturating_add(area.width as usize / 4),
                ubound_x.1.saturating_sub(area.width as usize / 4),
            );
            let clamped_y = cursor_index.1.clamp(
                ubound_y.0.saturating_add(area.height as usize / 2),
                ubound_y.1.saturating_sub(area.height as usize / 2),
            );

            for y in
                (clamped_y - (area.height / 2) as usize)..=(clamped_y + (area.height / 2) as usize)
            {
                let mut line: Line = Line::default();
                for x in clamped_x.saturating_sub((area.width / 4) as usize)
                    ..=clamped_x.saturating_add((area.width / 4) as usize)
                {
                    let span = if GridIndex(x, y) == cursor_index {
                        self.grid[GridIndex(x, y)]
                            .unwrap_or('.')
                            .to_string()
                            .black()
                            .on_white()
                    } else {
                        self.grid[GridIndex(x, y)].map_or_else(
                            || '.'.to_string().dark_gray(),
                            |letter: char| letter.to_string().green(),
                        )
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
