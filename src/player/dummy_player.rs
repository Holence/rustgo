use crate::{
    Coord, Stone,
    board::Board,
    player::{MoveAction, PlayerTrait},
};
use rand::{Rng, rngs::ThreadRng};

pub struct DummyPlayer {
    pub board: Board,
    pub rng: ThreadRng,
}

impl DummyPlayer {
    pub fn new(size: usize) -> Self {
        DummyPlayer {
            board: Board::new(size),
            rng: rand::rng(),
        }
    }
    pub fn random_coord(&mut self) -> Coord {
        // TODO random of usize???
        let idx = (self.rng.random::<u32>() % (self.board.size_2() as u32)) as usize;
        return self.board.coord(idx);
    }
}

impl PlayerTrait for DummyPlayer {
    fn play(&mut self, stone: Stone, coord: Coord) {
        self.board.place_stone(coord, stone).expect("should be ok");
    }

    fn genmove(&mut self, stone: Stone) -> MoveAction {
        // 随机生成坐标, 尝试几次
        for _ in 0..self.board.size() {
            let coord = self.random_coord();
            let result = self.board.place_stone(coord, stone);
            match result {
                // 如果成功
                Ok(_) => return MoveAction::Move { stone, coord },
                // 如果不成功, 则 continue
                Err(_) => {}
            }
        }
        // 如果都不成功, 则 PASS
        return MoveAction::Pass;
    }
}
