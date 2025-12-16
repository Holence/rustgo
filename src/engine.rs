use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

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

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}
