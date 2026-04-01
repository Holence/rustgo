use std::fmt::{Debug, Display};

// TODO translate 1-1 coord and A1 coord
#[derive(Clone, Copy)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Self {
        Coord { x, y }
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl Debug for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug is the same as Display
        Display::fmt(&self, f)
    }
}

impl Coord {
    /// A1-T19 (without 'I')
    pub fn from_a1(s: &str, size: usize) -> Option<Self> {
        if s.len() < 2 {
            return None;
        }
        let cs: &[u8] = s.as_bytes();
        let col = cs[0].to_ascii_uppercase();
        if !(b'A'..=b'T').contains(&col) || (col == b'I') {
            return None;
        }
        let x: usize = if col < b'I' {
            (col - b'A') as usize
        } else {
            // skip 'I'
            (col - b'A' - 1) as usize
        };

        let mut y: usize = 0;
        for c in &cs[1..] {
            if !c.is_ascii_digit() {
                return None;
            }
            y = y * 10 + (c - b'0') as usize;
        }
        y = size - y;

        return Some(Coord { x: x, y: y });
    }

    /// A1-T19 (without 'I')
    pub fn to_a1(&self, size: usize) -> Option<String> {
        if self.x >= 19 || self.y >= 19 {
            return None;
        }

        let mut col = (self.x as u8) + b'A';
        if col >= b'I' {
            col += 1;
        }
        let col = col as char;
        let row = size - self.y;

        Some(format!("{col}{row}"))
    }
}
