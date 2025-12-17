use crate::engine::Engine;
use crate::engine::PlaceStoneAction;
use crate::engine::Stone;

impl Stone {
    pub fn next(self) -> Stone {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
        }
    }
}

pub struct Game {
    engine: Engine,
    next_stone: Stone,
}

impl Game {
    pub fn new(size: usize) -> Self {
        Self {
            engine: Engine::new(size),
            next_stone: Stone::Black,
        }
    }

    pub fn size(&self) -> usize {
        self.engine.size()
    }

    pub fn board(&self) -> &Vec<Option<Stone>> {
        self.engine.board()
    }

    pub fn place_stone(&mut self, y: usize, x: usize) -> Result<PlaceStoneAction, &'static str> {
        let ret = self.engine.place_stone(y, x, self.next_stone)?;
        self.next_stone = self.next_stone.next();
        Ok(ret)
    }
}
