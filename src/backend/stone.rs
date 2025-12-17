use std::fmt::{Debug, Display, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stone {
    Black,
    White,
    Void,
}

impl Stone {
    #[inline]
    pub fn as_char(&self) -> char {
        match self {
            Stone::Black => '●',
            Stone::White => '○',
            Stone::Void => '_',
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Stone::Black => "●",
            Stone::White => "○",
            Stone::Void => "_",
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
