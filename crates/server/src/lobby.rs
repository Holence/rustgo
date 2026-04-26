use std::collections::HashMap;

use log::{error, info};
use tokio::sync::mpsc;

use crate::common::{ClientId, DownlinkMessage, ReqId, RoomId};

#[derive(Clone)]
pub enum LobbyMessage {
    Enter {
        client_id: ClientId,
        req_id: ReqId,
        tx: mpsc::Sender<DownlinkMessage>,
    },
    Chat {
        client_id: ClientId,
        content: String,
    },
    Quit {
        client_id: ClientId,
    },
}

pub struct LobbyActor {
    rx: mpsc::Receiver<LobbyMessage>,
    clients: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>,
    next_room_id: RoomId,
}

impl LobbyActor {
    pub fn new(rx: mpsc::Receiver<LobbyMessage>) -> Self {
        Self {
            rx,
            clients: HashMap::new(),
            next_room_id: 0,
        }
    }

    async fn send_to_session(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(tx) = self.clients.get(&client_id) {
            tx.send(msg).await.unwrap();
        } else {
            error!("client {client_id} not exist");
        }
    }
    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                LobbyMessage::Enter {
                    client_id,
                    req_id,
                    tx,
                } => {
                    self.clients.insert(client_id, tx);
                    self.send_to_session(
                        client_id,
                        DownlinkMessage::LobbyEnterAck {
                            req_id,
                            success: true,
                        },
                    )
                    .await;
                }
                LobbyMessage::Quit { client_id } => {
                    self.clients.remove(&client_id);
                }
                LobbyMessage::Chat { client_id, content } => {
                    info!("hear client[{}] says '{}'", client_id, content);
                    let msg = DownlinkMessage::LobbyChat { client_id, content };
                    for tx in self.clients.values() {
                        tx.send(msg.clone()).await.unwrap();
                    }
                }
            }
        }
    }
}
