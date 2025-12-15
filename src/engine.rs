use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy)]
pub enum Stone {
    Black,
    White,
}

/// 以左上角为原点，向下为+y，向右为+x
pub struct Board(Vec<Option<Stone>>);
// avoid writing `self.board.0`` in Engine
impl Deref for Board {
    type Target = Vec<Option<Stone>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
// avoid writing `self.board.0`` in Engine
impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Engine {
    width: usize,
    board: Board,
}
impl Engine {
    pub fn new(width: usize) -> Self {
        Engine {
            width: width,
            board: Board(vec![None; width * width]),
        }
    }

    pub fn xy_to_idx(&self, y: usize, x: usize) -> usize {
        debug_assert!(y < self.width);
        debug_assert!(x < self.width);
        return y * self.width + x;
    }

    pub fn place_stone(&mut self, y: usize, x: usize, stone: Stone) -> Result<(), &'static str> {
        let idx = self.xy_to_idx(y, x);
        debug_assert!(idx < self.board.len());

        // 禁止下到已有的棋子上
        if let Some(_) = self.board[idx] {
            return Err("禁止下到已有的棋子上");
        }
        // TODO 禁止使己方气尽

        // TODO 禁止全局同形

        self.board[idx] = Some(stone);
        Ok(())
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}
