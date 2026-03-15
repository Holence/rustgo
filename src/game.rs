use crate::{
    Coord, Stone,
    board::{Board, PlaceStoneResult},
};

pub struct Game {
    engine: Board,
    n_player: usize,
    n_stone: u8,
    cur_stone: Stone,
}

impl Game {
    pub fn new(size: usize, n_player: usize, n_stone: u8) -> Self {
        // TODO 传入 Vec<Box<dyn PlayerTrait>>, 每次落子时 player.genmove
        // TODO player附带阵营的信息, n_stone = 阵营数
        Self {
            engine: Board::new(size),
            n_player: n_player,
            n_stone: n_stone,
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
        self.cur_stone = self.cur_stone.next_stone(self.n_stone);
        Ok(ret)
    }
}
