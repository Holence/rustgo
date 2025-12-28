use crate::backend::{Board, Coord, PlaceStoneResult, Stone};

pub struct Game {
    engine: Board,
    n_player: usize,
    cur_stone: Stone,
}

impl Game {
    pub fn new(size: usize, n_player: usize) -> Self {
        Self {
            engine: Board::new(size),
            n_player: n_player,
            cur_stone: Stone::BLACK,
        }
    }

    pub fn size(&self) -> usize {
        self.engine.size()
    }

    pub fn board(&self) -> &[Stone] {
        self.engine.board()
    }

    pub fn place_stone(&mut self, coord: Coord) -> PlaceStoneResult {
        let ret = self.engine.place_stone(coord, self.cur_stone)?;
        self.cur_stone = self.cur_stone.next_stone(self.n_player);
        Ok(ret)
    }
}
