use tokio::sync::mpsc::{Receiver, Sender};

use crate::player::{PlayerId, PlayerMessage, PlayerTrait, ServerMessage};

// 用channel连接的Player，仅用于对接GUI前端
pub struct ChannelPlayer {
    player_id: PlayerId,
    downlink_to_ui: Sender<ServerMessage>,
    uplink_from_ui: Receiver<PlayerMessage>,
}

impl ChannelPlayer {
    pub fn new(
        player_id: PlayerId,
        downlink_to_ui: Sender<ServerMessage>,
        uplink_from_ui: Receiver<PlayerMessage>,
    ) -> Self {
        ChannelPlayer {
            player_id,
            downlink_to_ui,
            uplink_from_ui,
        }
    }
}

impl PlayerTrait for ChannelPlayer {
    fn run(mut self, uplink_tx: Sender<PlayerMessage>, mut downlink_rx: Receiver<ServerMessage>) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = downlink_rx.recv() => {
                        self.downlink_to_ui.send(msg).await.unwrap();
                    }
                    Some(msg) = self.uplink_from_ui.recv() => {
                        uplink_tx.send(msg).await.unwrap();
                    }
                };
            }
        });
    }

    fn player_id(&self) -> super::PlayerId {
        self.player_id
    }
}
