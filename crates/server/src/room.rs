use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::common::{ClientId, DownlinkMessage};

#[derive(Clone)]
pub enum RoomMessage {
    // TODO RoomInfo(RoomId, Vec<TeamInfo>) // downlink only
    Enter(ClientId, mpsc::Sender<DownlinkMessage>),
    RoomChat(ClientId, String),
    // TODO CreateTeam(PlayerId)
    // TODO JoinTeam(PlayerId, TeamId)
    // TODO LeaveTeam(PlayerId, TeamId)
    Quit(ClientId),
}

pub struct RoomActor {
    rx: mpsc::Receiver<RoomMessage>,
    room_name: String,
    host: ClientId, // 房主

    clients_tx: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>,
}

impl RoomActor {
    pub fn new(rx: mpsc::Receiver<RoomMessage>, room_name: String, host: ClientId) -> Self {
        Self {
            rx,
            room_name,
            host,
            clients_tx: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                RoomMessage::Enter(_, sender) => todo!(),
                RoomMessage::RoomChat(_, _) => todo!(),
                RoomMessage::Quit(_) => todo!(),
            }
        }
    }
}
