use std::collections::HashMap;

use log::error;
use tokio::sync::mpsc;

use crate::common::{ChatRecord, ClientId, DownlinkMessage, ReqId, RoomId};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ClientRecord {
    username: String,
    team: u64,
}

#[derive(Clone)]
pub enum RoomMessage {
    Enter {
        req_id: ReqId,
        client_id: ClientId,
        username: String,
        client_tx: mpsc::Sender<DownlinkMessage>,
    },
    RoomChat {
        client_id: ClientId,
        content: String,
    },
    // TODO CreateTeam(PlayerId)
    // TODO JoinTeam(PlayerId, TeamId)
    // TODO LeaveTeam(PlayerId, TeamId)
    Quit(ClientId),
}

pub struct RoomActor {
    rx: mpsc::Receiver<RoomMessage>,
    room_id: RoomId,
    room_name: String,
    host_id: ClientId, // 房主

    clients: HashMap<ClientId, ClientRecord>,
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
            ClientRecord {
                username: host_username,
                team: 0,
            },
        );
        let mut clients_tx = HashMap::new();
        clients_tx.insert(host_id, host_tx);
        Self {
            rx,
            room_id,
            room_name,
            host_id,
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

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                RoomMessage::Enter {
                    req_id,
                    client_id,
                    username,
                    client_tx,
                } => {
                    if self.clients.contains_key(&client_id) {
                        error!("client[{}] already exists", client_id);
                        continue;
                    }
                    assert!(!self.clients_tx.contains_key(&client_id));

                    self.clients
                        .insert(client_id, ClientRecord { username, team: 0 });
                    self.clients_tx.insert(client_id, client_tx);

                    self.send_to_client(
                        client_id,
                        DownlinkMessage::RoomEnterAck {
                            req_id,
                            success: true,
                            room_id: self.room_id,
                            chats: self.chats.clone(),
                        },
                    )
                    .await;

                    // TODO boardcast ClientRecord
                }
                RoomMessage::RoomChat { client_id, content } => {
                    self.chats.push(ChatRecord {
                        client_id,
                        username: self.client_username_cloned(client_id),
                        content: content.clone(),
                    });

                    self.broadcast(DownlinkMessage::RoomChat {
                        room_id: self.room_id,
                        client_id,
                        username: self.client_username_cloned(client_id),
                        content,
                    })
                    .await;
                }
                RoomMessage::Quit(client_id) => {
                    self.clients.remove(&client_id).unwrap();
                    self.clients_tx.remove(&client_id).unwrap();
                }
            }
        }
    }
}
