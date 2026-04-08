use crate::{PlayerMessage, ServerMessage, team::TeamId};
use tokio::sync::mpsc::{Receiver, Sender};

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
    pub player_name: String,

    /// server -> player
    pub downlink_tx: Sender<ServerMessage>,
}
impl PlayerHandle {
    pub fn new(
        player_id: PlayerId,
        player_name: String,
        downlink_tx: Sender<ServerMessage>,
    ) -> Self {
        Self {
            player_id,
            player_name,
            downlink_tx,
        }
    }

    pub async fn send(&self, msg: ServerMessage) {
        self.downlink_tx.send(msg).await.unwrap();
    }
}

pub trait PlayerTrait {
    fn run(
        self,
        player_id: PlayerId,
        uplink_tx: Sender<PlayerMessage>,
        downlink_rx: Receiver<ServerMessage>,
    );
}

pub mod channel_player;
pub mod dummy_player;
pub mod local_gnugo_player;
