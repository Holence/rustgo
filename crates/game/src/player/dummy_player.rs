use std::{thread::sleep, time::Duration};

use crate::player::{MoveAction, PlayerError, PlayerTrait};
use rand::{RngExt, rngs::ThreadRng};
use rustgo::{Coord, Stone, board::Board};

pub struct DummyPlayer {
    board: Board,
    rng: ThreadRng,
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
        let idx = (self.rng.random::<u32>() % (self.board.size_square() as u32)) as usize;
        return self.board.coord(idx);
    }
}

impl PlayerTrait for DummyPlayer {
    fn play(&mut self, move_action: MoveAction) -> Result<(), PlayerError> {
        match move_action {
            MoveAction::Move { stone, coord } => match self.board.place_stone(coord, stone) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("Try to place {stone} at {coord}, but board err: {e}");
                    panic!()
                }
            },
            MoveAction::Pass => todo!(),
            MoveAction::Resign => todo!(),
        }
    }

    fn genmove(&mut self, stone: Stone) -> Result<MoveAction, PlayerError> {
        // 随机生成坐标, 尝试几次
        sleep(Duration::from_micros(500));
        for _ in 0..self.board.size() {
            let coord = self.random_coord();
            let result = self.board.place_stone(coord, stone);
            if result.is_ok() {
                return Ok(MoveAction::Move { stone, coord });
            }
        }
        // 如果都不成功, 则 PASS
        return Ok(MoveAction::Pass);
    }
}
