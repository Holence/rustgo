use crate::{Coord, Stone};

#[derive(Debug)]
pub enum PlayerError {
    IoError(std::io::Error),
    EngineError(String),
}

impl From<std::io::Error> for PlayerError {
    fn from(value: std::io::Error) -> Self {
        PlayerError::IoError(value)
    }
}

#[derive(Clone, Copy)]
pub enum MoveAction {
    Move { stone: Stone, coord: Coord },
    Pass,
    Resign,
}

pub trait PlayerTrait {
    /// others placed `stone` at `coord`
    /// self should acknowledge this info
    fn play(&mut self, move_action: MoveAction) -> Result<(), PlayerError>;

    /// generate `MoveAction` for `stone`
    /// it's self turn to move
    fn genmove(&mut self, stone: Stone) -> Result<MoveAction, PlayerError>;
}

pub mod dummy_player;
pub mod local_gnugo_player;
