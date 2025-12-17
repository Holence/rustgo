use std::fmt::{Debug, Display, Write};

// TODO a coord mod
// TODO translate 1-1 coord and A1 coord
pub struct Coord {
    y: usize,
    x: usize,
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

pub struct PlaceStoneAction {
    pub place: Coord,      // 落子坐标
    pub eaten: Vec<Coord>, // 吃子坐标
}

#[derive(Clone, Copy)]
pub enum Stone {
    Black,
    White,
}
impl Stone {
    #[inline]
    pub fn as_char(&self) -> char {
        match self {
            Stone::Black => '●',
            Stone::White => '○',
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
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
        Display::fmt(self, f)
    }
}

/// 以左上角为原点，向下为+y，向右为+x
type Board = Vec<Option<Stone>>;

pub struct Engine {
    size: usize,
    board: Board,
}
impl Engine {
    pub fn new(size: usize) -> Self {
        Engine {
            size: size,
            board: vec![None; size * size],
        }
    }

    pub fn xy_to_idx(&self, y: usize, x: usize) -> usize {
        debug_assert!(y < self.size);
        debug_assert!(x < self.size);
        return y * self.size + x;
    }

    pub fn place_stone(
        &mut self,
        y: usize,
        x: usize,
        stone: Stone,
    ) -> Result<PlaceStoneAction, &'static str> {
        let idx = self.xy_to_idx(y, x);
        debug_assert!(idx < self.board.len());

        // 禁止下到已有的棋子上
        if let Some(_) = self.board[idx] {
            return Err("禁止下到已有的棋子上");
        }
        // TODO 禁止使己方气尽

        // TODO 禁止全局同形

        self.board[idx] = Some(stone);
        Ok(PlaceStoneAction {
            place: Coord { y, x },
            eaten: vec![], // TODO eaten
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_string(&self) -> String {
        let mut s = String::with_capacity(self.size * self.size * 2);
        let mut idx = 0;
        for _ in 0..self.size {
            for _ in 0..self.size {
                let ch = match self.board[idx] {
                    Some(stone) => stone.as_char(),
                    None => '_',
                };
                s.push(ch);
                s.push(' ');
                idx += 1;
            }
            s.push('\n');
        }
        return s;
    }
}
