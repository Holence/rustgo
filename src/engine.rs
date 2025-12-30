use crate::{Coord, Stone};

enum MoveAction {
    Move { stone: Stone, coord: Coord },
    Pass,
    Resign,
}

trait EngineTrait {
    // place `stone` at `coord`
    fn play(&mut self, stone: Stone, coord: Coord);

    /// generate `MoveAction` for `stone`
    fn genmove(&mut self, stone: Stone) -> MoveAction;
}

mod dummy_engine;
