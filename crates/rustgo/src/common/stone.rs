use std::fmt::{Debug, Display, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Stone(u8);

static LUT: &[char] = &['_', '●', '○', '$', '#', '&', '@'];

impl Stone {
    pub const VOID: Stone = Stone(0);
    pub const BLACK: Stone = Stone(1);
    pub const WHITE: Stone = Stone(2);

    pub fn new(x: u8) -> Self {
        Stone(x)
    }

    #[inline]
    pub fn as_char(&self) -> char {
        LUT[self.0 as usize]
    }

    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// 在`n`色棋的棋局中，计算出当前棋子的下一种棋子
    pub fn next_stone(self, n: u8) -> Self {
        debug_assert!(self != Stone::VOID);
        debug_assert!(n <= LUT.len() as u8);
        if (self.0) < n {
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
