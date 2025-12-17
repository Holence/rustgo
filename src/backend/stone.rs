use std::fmt::{Debug, Display, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stone {
    Void,
    Black,
    White,
}

impl Stone {
    #[inline]
    pub fn as_char(&self) -> char {
        match self {
            Stone::Void => '_',
            Stone::Black => '●',
            Stone::White => '○',
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Stone::Void => "_",
            Stone::Black => "●",
            Stone::White => "○",
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
