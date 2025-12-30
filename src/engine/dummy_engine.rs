use crate::{
    Coord, Stone,
    board::Board,
    engine::{EngineTrait, MoveAction},
};
use rand::{Rng, rngs::ThreadRng};

pub struct DummyEngine {
    pub board: Board,
    pub rng: ThreadRng,
}

impl DummyEngine {
    pub fn new(size: usize) -> Self {
        DummyEngine {
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

impl EngineTrait for DummyEngine {
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
