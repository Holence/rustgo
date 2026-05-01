use std::collections::HashMap;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::{
    common::{ClientId, DownlinkMessage, RoomId, UplinkMessage},
    room::{RoomActor, RoomMessage},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RoomInfoUpdate {
    ChangeName { room_id: RoomId, room_name: String },
    // ChangeState { room_id: RoomId } // state: Teaming/GameStart/GameEnd
}

struct RoomInfo {
    room_tx: mpsc::Sender<RoomMessage>,
    room_name: String,
    // state: Teaming/GameStart/GameEnd
}

enum ClientLocation {
    AtLobby,
    AtRoom(RoomId),
}

struct ClientInfo {
    client_tx: mpsc::Sender<DownlinkMessage>,
    // client_name: String,
    location: ClientLocation,
}

pub enum LobbyMessage {
    RegisterClient {
        client_id_tx: oneshot::Sender<ClientId>,
        client_tx: mpsc::Sender<DownlinkMessage>,
    },
    UnregisterClient {
        client_id: ClientId,
    },
    ClientMessage {
        msg: UplinkMessage,
    },
    RoomInfoUpdate {
        info: RoomInfoUpdate,
    },
}

pub struct LobbyActor {
    rx: mpsc::Receiver<LobbyMessage>,
    clients: HashMap<ClientId, ClientInfo>,
    rooms: HashMap<RoomId, RoomInfo>,
    next_client_id: ClientId,
    next_room_id: RoomId,
}

impl LobbyActor {
    pub fn new(rx: mpsc::Receiver<LobbyMessage>) -> Self {
        Self {
            rx,
            clients: HashMap::new(),
            rooms: HashMap::new(),
            next_client_id: 0,
            next_room_id: 0,
        }
    }

    async fn send_to_client(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(client_info) = self.clients.get(&client_id) {
            client_info.client_tx.send(msg).await.unwrap();
        } else {
            error!("client {client_id} not exist");
        }
    }

    async fn handle_msg(&mut self, msg: UplinkMessage) {
        match msg {
            UplinkMessage::Ping { client_id, req_id } => todo!(),
            UplinkMessage::Quit { client_id } => {
                self.clients.remove(&client_id);
            }
            UplinkMessage::LobbyChat { client_id, content } => {
                if !self.clients.contains_key(&client_id) {
                    error!("client[{client_id}] not exist");
                    return;
                }

                info!("hear client[{client_id}] says '{content}'");
                let msg = DownlinkMessage::LobbyChat { client_id, content };
                for client_info in self.clients.values() {
                    client_info.client_tx.send(msg.clone()).await.unwrap();
                }
            }
            UplinkMessage::LobbyCreateRoom {
                client_id,
                req_id,
                room_name,
            } => {
                if !self.clients.contains_key(&client_id) {
                    error!("client[{client_id}] not exist");
                    return;
                }

                let (room_tx, room_rx) = mpsc::channel(32);
                let room_actor = RoomActor::new(room_rx, room_name.clone(), client_id);
                tokio::spawn(room_actor.run());

                let room_id = self.next_room_id;
                self.rooms.insert(room_id, RoomInfo { room_tx, room_name });

                self.send_to_client(
                    client_id,
                    DownlinkMessage::LobbyCreateRoomAck {
                        req_id,
                        room_id: Some(room_id),
                    },
                )
                .await;

                self.next_room_id += 1;
            }
            UplinkMessage::RoomEnter {
                client_id,
                req_id,
                room_id,
            } => todo!(),
            UplinkMessage::RoomChat {
                client_id,
                room_id,
                content,
            } => todo!(),
            UplinkMessage::RoomQuit {
                client_id,
                req_id,
                room_id,
            } => todo!(),
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                LobbyMessage::RegisterClient {
                    client_id_tx,
                    client_tx,
                } => {
                    let client_id = self.next_client_id;
                    self.clients.insert(
                        client_id,
                        ClientInfo {
                            client_tx,
                            location: ClientLocation::AtLobby,
                        },
                    );

                    client_id_tx.send(client_id).unwrap();
                    self.send_to_client(client_id, DownlinkMessage::Greeting { client_id })
                        .await;

                    self.next_client_id += 1;
                }
                LobbyMessage::UnregisterClient { client_id } => {
                    let Some(client_info) = self.clients.remove(&client_id) else {
                        error!("client[{}] not exists", client_id);
                        continue;
                    };

                    match client_info.location {
                        ClientLocation::AtLobby => todo!(),
                        ClientLocation::AtRoom(_) => todo!(),
                    }
                }
                LobbyMessage::ClientMessage { msg } => self.handle_msg(msg).await,
                LobbyMessage::RoomInfoUpdate { info } => todo!(),
            }
        }
    }
}
