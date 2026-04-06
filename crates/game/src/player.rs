use rustgo::{Coord, Stone};
use tokio::sync::mpsc::{self, Sender, error::SendError};

use crate::{Action, PlayerMessage, ServerMessage, team::TeamId};

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

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct PlayerId(usize);
impl PlayerId {
    pub fn new(id: usize) -> Self {
        PlayerId(id)
    }
}
#[derive(Clone, Debug)]
pub struct PlayerInfo {
    pub player_id: PlayerId,
    pub team_id: TeamId,
    pub player_name: String,
    pub eaten_stones: usize,
    pub time_left: usize,
}

pub struct PlayerHandle {
    pub player_id: PlayerId,

    /// server -> player
    pub downlink_tx: Sender<ServerMessage>,
}
impl PlayerHandle {
    pub async fn send(&self, msg: ServerMessage) {
        self.downlink_tx.send(msg).await.unwrap();
    }
}

pub trait PlayerTrait {
    fn spawn(self, player_id: PlayerId, uplink_tx: Sender<PlayerMessage>) -> PlayerHandle;

    /// others placed `stone` at `coord`
    /// self should acknowledge this info
    fn play(&mut self, stone: Stone, coord: Coord) -> Result<(), PlayerError>;

    /// Player只返回落子选择, 不能修改自身的棋盘状态, 该落子是否合法需要得到服务器的确认 ServerMessage::PlayerMove 才算落子成功
    /// (所以GnuGo里不应该用 genmove, 而应该用 reg_genmove)
    fn genmove(&mut self, stone: Stone) -> Result<Action, PlayerError>;
}
#[cfg(feature = "broken")]
pub mod channel_player;
pub mod dummy_player;
pub mod local_gnugo_player;
