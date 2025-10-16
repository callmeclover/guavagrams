use std::{
    fmt::Display,
    ops::{Add, AddAssign, Index, IndexMut, Sub},
};

use super::{Grid};

/// A XY coordinate on a 2D grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Coordinate(pub i8, pub i8);

impl Add for Coordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // We use `saturating_add` to clip the grid. (127 + 1 = 127)
        Self(self.0.saturating_add(rhs.0), self.1.saturating_add(rhs.1))
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[allow(clippy::cast_possible_wrap)]
impl From<GridIndex> for Coordinate {
    fn from(value: GridIndex) -> Self {
        Self(
            (-128_i8).wrapping_add_unsigned(value.0),
            (-128_i8).wrapping_sub_unsigned(value.1),
        )
    }
}

impl Sub for Coordinate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // We use `saturating_sub` to clip the grid. (-128 - 1 = -128)
        Self(self.0.saturating_sub(rhs.0), self.1.saturating_sub(rhs.1))
    }
}

impl<T> Index<Coordinate> for Grid<T> {
    type Output = T;

    fn index(&self, index: Coordinate) -> &Self::Output {
        &self[GridIndex::from(index)]
    }
}

impl<T> IndexMut<Coordinate> for Grid<T> {
    fn index_mut(&mut self, index: Coordinate) -> &mut Self::Output {
        &mut self[GridIndex::from(index)]
    }
}

impl Coordinate {
    /// Calculates self + rhs.
    ///
    /// Returns a tuple of the addition along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (x, x_overflowed) = self.0.overflowing_add(rhs.0);
        let (y, y_overflowed) = self.1.overflowing_add(rhs.1);
        (Self(x, y), x_overflowed || y_overflowed)
    }
}

/// An index to help with indexing `Grid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridIndex(pub u8, pub u8);

#[allow(clippy::cast_sign_loss)]
impl From<Coordinate> for GridIndex {
    fn from(value: Coordinate) -> Self {
        Self(
            128_u8.wrapping_add_signed(value.0),
            128_u8.wrapping_sub_signed(value.1),
        )
    }
}

impl Add for GridIndex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0), self.1.saturating_add(rhs.1))
    }
}

impl Sub for GridIndex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0), self.1.saturating_sub(rhs.1))
    }
}

impl<T> Index<GridIndex> for Grid<T> {
    type Output = T;

    fn index(&self, index: GridIndex) -> &Self::Output {
        &self.0[index.1 as usize][index.0 as usize]
    }
}

impl<T> IndexMut<GridIndex> for Grid<T> {
    fn index_mut(&mut self, index: GridIndex) -> &mut Self::Output {
        &mut self.0[index.1 as usize][index.0 as usize]
    }
}
