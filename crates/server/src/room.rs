use std::collections::HashMap;

use log::error;
use rustgo::Stone;
use tokio::sync::mpsc;

use crate::common::{ChatRecord, ClientId, DownlinkMessage, ReqId, RoomId};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RoomClientRecord {
    pub username: String,

    /// each team corresponds to one type of Stone
    ///
    /// if `None`, then this client is not in any team, it is a Spectator
    pub team: Option<Stone>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RoomClientAction {
    Enter,
    Change,
    Quit,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum GameState {
    Teaming,
    Ongoing,  // TODO attach board state
    Finished, // TODO attach board state
}

#[derive(Clone)]
pub enum RoomMessage {
    Enter {
        client_id: ClientId,
        req_id: ReqId,
        username: String,
        client_tx: mpsc::Sender<DownlinkMessage>,
    },
    ChangeTeam {
        client_id: ClientId,
        req_id: ReqId,
        team: Option<Stone>,
    },
    Chat {
        client_id: ClientId,
        content: String,
    },
    Quit(ClientId),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RoomSnapshot {
    pub room_id: RoomId,
    pub room_name: String,
    pub host_id: ClientId,
    pub state: GameState,
    pub clients: HashMap<ClientId, RoomClientRecord>,
    pub chats: Vec<ChatRecord>,
}

pub struct RoomActor {
    rx: mpsc::Receiver<RoomMessage>,
    room_id: RoomId,
    room_name: String,
    host_id: ClientId, // 房主
    state: GameState,

    clients: HashMap<ClientId, RoomClientRecord>,
    clients_tx: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>,

    chats: Vec<ChatRecord>,
}

impl RoomActor {
    pub fn new(
        rx: mpsc::Receiver<RoomMessage>,
        room_id: RoomId,
        room_name: String,
        host_id: ClientId,
        host_username: String,
        host_tx: mpsc::Sender<DownlinkMessage>,
    ) -> Self {
        let mut clients = HashMap::new();
        clients.insert(
            host_id,
            RoomClientRecord {
                username: host_username,
                team: None,
            },
        );
        let mut clients_tx = HashMap::new();
        clients_tx.insert(host_id, host_tx);
        Self {
            rx,
            room_id,
            room_name,
            host_id,
            state: GameState::Teaming,
            clients,
            clients_tx,
            chats: vec![],
        }
    }

    fn client_username_cloned(&self, client_id: ClientId) -> String {
        self.clients.get(&client_id).unwrap().username.clone()
    }

    async fn send_to_client(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(client_tx) = self.clients_tx.get(&client_id) {
            client_tx.send(msg).await.unwrap();
        } else {
            error!("client[{client_id}] not exist");
        }
    }

    async fn broadcast(&self, msg: DownlinkMessage) {
        for client_tx in self.clients_tx.values() {
            client_tx.send(msg.clone()).await.unwrap();
        }
    }

    pub fn get_snapshot(&self) -> RoomSnapshot {
        RoomSnapshot {
            room_id: self.room_id,
            room_name: self.room_name.clone(),
            host_id: self.host_id,
            state: self.state.clone(),
            clients: self.clients.clone(),
            chats: self.chats.clone(),
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                RoomMessage::Enter {
                    client_id,
                    req_id,
                    username,
                    client_tx,
                } => {
                    if self.clients.contains_key(&client_id) {
                        error!("client[{}] already exists", client_id);
                        continue;
                    }
                    assert!(!self.clients_tx.contains_key(&client_id));

                    self.clients.insert(
                        client_id,
                        RoomClientRecord {
                            username,
                            team: None,
                        },
                    );
                    self.clients_tx.insert(client_id, client_tx);

                    self.send_to_client(
                        client_id,
                        DownlinkMessage::RoomEnterAck {
                            req_id,
                            success: true,
                            room_snapshot: self.get_snapshot(),
                        },
                    )
                    .await;

                    // TODO boardcast RoomClientUpdate
                }
                RoomMessage::ChangeTeam {
                    client_id,
                    req_id,
                    team,
                } => todo!(),
                RoomMessage::Chat { client_id, content } => {
                    // TODO ensure_client_in_room

                    self.chats.push(ChatRecord {
                        client_id,
                        username: self.client_username_cloned(client_id),
                        content: content.clone(),
                    });

                    self.broadcast(DownlinkMessage::RoomChatUpdate {
                        room_id: self.room_id,
                        client_id,
                        username: self.client_username_cloned(client_id),
                        content,
                    })
                    .await;
                }
                RoomMessage::Quit(client_id) => {
                    // TODO ensure_client_in_room

                    self.clients.remove(&client_id);
                    self.clients_tx.remove(&client_id);

                    // TODO boardcast RoomClientUpdate
                }
            }
        }
    }
}
