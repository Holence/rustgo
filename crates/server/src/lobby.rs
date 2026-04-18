use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::common::{ClientId, DownlinkMessage};

#[derive(Clone)]
pub enum LobbyMessage {
    Enter(ClientId, mpsc::Sender<DownlinkMessage>),
    Chat(ClientId, String),
    Quit(ClientId),
}

pub struct LobbyActor {
    rx: mpsc::Receiver<LobbyMessage>,
    clients: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>,
}

impl LobbyActor {
    pub fn new(rx: mpsc::Receiver<LobbyMessage>) -> Self {
        Self {
            rx,
            clients: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                LobbyMessage::Enter(client_id, tx) => {
                    self.clients.insert(client_id, tx);
                }
                LobbyMessage::Quit(client_id) => {
                    self.clients.remove(&client_id);
                }
                LobbyMessage::Chat(client_id, s) => {
                    println!("[lobby] hear client[{}] says '{}'", client_id, s);
                    let msg = DownlinkMessage::LobbyChat(client_id, s);
                    for tx in self.clients.values() {
                        let _ = tx.send(msg.clone()).await;
                    }
                }
            }
        }
    }
}
