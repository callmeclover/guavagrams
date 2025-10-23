#[derive(Debug, PartialEq, Eq)]
pub struct GridSize {
    pub width: usize,
    pub height: usize,
    pub bound_x: Bound,
    pub bound_y: Bound,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Bound(pub isize, pub isize);

impl GridSize {
    pub const fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            bound_x: Bound::new(width),
            bound_y: Bound::new(height),
        }
    }
}

impl Bound {
    pub const fn new(full: usize) -> Self {
        let middle: isize = (full / 2).cast_signed();

        if full.is_multiple_of(2) {
            Self(-middle, middle - 1)
        } else {
            Self(-middle, middle)
        }
    }

    pub const fn as_unsigned(&self) -> (usize, usize) {
        if -self.0 == self.1 {
            (0, self.1.cast_unsigned() * 2)
        } else {
            (0, self.1.cast_unsigned() * 2 + 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Bound;

    #[test]
    fn test_bound_correctness() {
        // Where `full` is 256, `bound` should be (-128, 127)
        assert_eq!(Bound::new(256), Bound(-128, 127));

        // Where `full` is 257, `bound` should be (-128, 128)
        assert_eq!(Bound::new(257), Bound(-128, 128));

        // Where `full` is 1000, `bound` should be (-500, 499)
        assert_eq!(Bound::new(1000), Bound(-500, 499));

        // Where `full` is 8, `bound` should be (-4, 3)
        assert_eq!(Bound::new(8), Bound(-4, 3));
    }

    #[test]
    fn test_unsigned_bound_correctness() {
        // Where `full` is 256, `bound` should be 255
        assert_eq!(Bound::new(256).as_unsigned(), (0, 255));

        // Where `full` is 257, `bound` should be 256
        assert_eq!(Bound::new(257).as_unsigned(), (0, 256));

        // Where `full` is 1000, `bound` should be 999
        assert_eq!(Bound::new(1000).as_unsigned(), (0, 999));

        // Where `full` is 8, `bound` should be 7
        assert_eq!(Bound::new(8).as_unsigned(), (0, 7));
    }
}
