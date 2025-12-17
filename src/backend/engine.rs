use crate::{backend::Coord, backend::Stone};

pub struct PlaceStoneResult {
    pub eaten: Vec<Coord>, // 吃子坐标
}

pub type EngineResult = Result<PlaceStoneResult, &'static str>;

pub struct Engine {
    size: usize,
    board: Box<[Option<Stone>]>, // 以左上角为原点，向下为+y，向右为+x
}

impl Engine {
    pub fn new(size: usize) -> Self {
        Engine {
            size: size,
            board: vec![None; size * size].into_boxed_slice(),
        }
    }

    fn idx(&self, coord: Coord) -> usize {
        debug_assert!(coord.y < self.size);
        debug_assert!(coord.x < self.size);
        return coord.y * self.size + coord.x;
    }

    pub fn place_stone(&mut self, coord: Coord, stone: Stone) -> EngineResult {
        let idx = self.idx(coord);
        debug_assert!(idx < self.board.len());

        // 禁止下到已有的棋子上
        if let Some(_) = self.board[idx] {
            return Err("禁止下到已有的棋子上");
        }
        // TODO 禁止使己方气尽

        // TODO 禁止全局同形

        self.board[idx] = Some(stone);
        Ok(PlaceStoneResult {
            eaten: vec![], // TODO eaten
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn board(&self) -> &[Option<Stone>] {
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
