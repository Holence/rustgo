use std::{fmt::Display, usize};

// TODO a coord mod
// TODO translate 1-1 coord and A1 coord
pub struct Coord {
    y: usize,
    x: usize,
}

impl Coord {
    pub fn new(y: usize, x: usize) -> Self {
        Coord { y, x }
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}
