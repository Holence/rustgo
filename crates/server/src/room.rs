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
    host_id: ClientId, // 房主

    clients_tx: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>,
}

impl RoomActor {
    pub fn new(
        rx: mpsc::Receiver<RoomMessage>,
        room_name: String,
        host_id: ClientId,
        host_tx: mpsc::Sender<DownlinkMessage>,
    ) -> Self {
        let mut clients_tx = HashMap::new();
        clients_tx.insert(host_id, host_tx);
        Self {
            rx,
            room_name,
            host_id,
            clients_tx,
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                RoomMessage::Enter(client_id, sender) => {}
                RoomMessage::RoomChat(_, _) => todo!(),
                RoomMessage::Quit(client_id) => {
                    self.clients_tx.remove(&client_id).unwrap();
                }
            }
        }
    }
}
