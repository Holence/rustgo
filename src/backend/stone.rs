use std::fmt::{Debug, Display, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Stone(u8);

static LUT: &[char] = &['_', '●', '○', '$', '#', '&', '@'];

impl Stone {
    pub const VOID: Stone = Stone(0);
    pub const BLACK: Stone = Stone(1);
    pub const WHITE: Stone = Stone(2);

    #[inline]
    pub fn as_char(&self) -> char {
        LUT[self.0 as usize]
    }

    pub fn next_stone(self, n_player: usize) -> Self {
        debug_assert!(self != Stone::VOID);
        debug_assert!(n_player <= LUT.len());
        if (self.0 as usize) < n_player {
            return Stone(self.0 + 1);
        } else {
            return Stone::BLACK;
        }
    }
}

impl Display for Stone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.as_char())
    }
}

impl Debug for Stone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug is the same as Display
        Display::fmt(&self, f)
    }
}
