use std::collections::HashMap;

use log::error;
use tokio::sync::mpsc;

use crate::common::{ClientId, DownlinkMessage, ReqId, RoomId};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ClientRecord {
    // name
    team: u64,
}

#[derive(Clone)]
pub enum RoomMessage {
    // TODO RoomInfo(RoomId, Vec<TeamInfo>) // downlink only
    Enter {
        req_id: ReqId,
        client_id: ClientId,
        client_tx: mpsc::Sender<DownlinkMessage>,
    },
    RoomChat(ClientId, String),
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
}

impl RoomActor {
    pub fn new(
        rx: mpsc::Receiver<RoomMessage>,
        room_id: RoomId,
        room_name: String,
        host_id: ClientId,
        host_tx: mpsc::Sender<DownlinkMessage>,
    ) -> Self {
        let mut clients = HashMap::new();
        clients.insert(host_id, ClientRecord { team: 0 });
        let mut clients_tx = HashMap::new();
        clients_tx.insert(host_id, host_tx);
        Self {
            rx,
            room_id,
            room_name,
            host_id,
            clients,
            clients_tx,
        }
    }

    async fn send_to_client(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(client_tx) = self.clients_tx.get(&client_id) {
            client_tx.send(msg).await.unwrap();
        } else {
            error!("client[{client_id}] not exist");
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                RoomMessage::Enter {
                    req_id,
                    client_id,
                    client_tx,
                } => {
                    if self.clients.contains_key(&client_id) {
                        error!("client[{}] already exists", client_id);
                        continue;
                    }
                    assert!(!self.clients_tx.contains_key(&client_id));

                    self.clients.insert(client_id, ClientRecord { team: 0 });
                    self.clients_tx.insert(client_id, client_tx);

                    self.send_to_client(
                        client_id,
                        DownlinkMessage::RoomEnterAck {
                            req_id,
                            success: true,
                            room_id: self.room_id,
                        },
                    )
                    .await;

                    // TODO boardcast ClientRecord
                }
                RoomMessage::RoomChat(_, _) => todo!(),
                RoomMessage::Quit(client_id) => {
                    self.clients_tx.remove(&client_id).unwrap();
                }
            }
        }
    }
}
