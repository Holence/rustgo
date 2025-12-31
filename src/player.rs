use crate::{Coord, Stone};

enum MoveAction {
    Move { stone: Stone, coord: Coord },
    Pass,
    Resign,
}

trait PlayerTrait {
    /// others placed `stone` at `coord`
    /// self should acknowledge this info
    fn play(&mut self, stone: Stone, coord: Coord);

    /// generate `MoveAction` for `stone`
    /// it's self turn to move
    fn genmove(&mut self, stone: Stone) -> MoveAction;
}

mod dummy_player;
