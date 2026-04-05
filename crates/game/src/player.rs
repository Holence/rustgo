use rustgo::{Coord, Stone};

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

pub enum GameMessage {
    MoveAction(MoveAction),
    GenMove(Stone),
    GameOver,
}

pub trait PlayerTrait {
    /// others placed `stone` at `coord`
    /// self should acknowledge this info
    fn play(&mut self, move_action: MoveAction) -> Result<(), PlayerError>;

    /// generate `MoveAction` for `stone`
    /// it's self turn to move
    fn genmove(&mut self, stone: Stone) -> Result<MoveAction, PlayerError>;

    // TODO notify current player
    // TODO notify GameOverInfo
    // 因为多色棋、联棋的对局结束得由Game来决定, 每个Player是不知道是否可以结束的
}

pub mod channel_player;
pub mod dummy_player;
pub mod local_gnugo_player;
