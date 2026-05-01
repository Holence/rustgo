use std::collections::HashMap;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::{
    common::{ClientId, DownlinkMessage, RoomId, UplinkMessage},
    room::{RoomActor, RoomMessage},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatRecord {
    pub client_id: ClientId,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RoomRecord {
    pub room_name: String,
    // room_state // state: Teaming/GameStart/GameEnd
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum ClientLocation {
    AtLobby,
    AtRoom(RoomId),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ClientRecord {
    location: ClientLocation,
    // name
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LobbyPartialInfo {
    Chat(ChatRecord),
    Room {
        room_id: RoomId,
        room_record: RoomRecord,
    },
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
    InfoUpdate {
        info: LobbyPartialInfo,
    },
}

pub struct LobbyActor {
    rx: mpsc::Receiver<LobbyMessage>,

    clients: HashMap<ClientId, ClientRecord>,
    clients_tx: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>,

    rooms: HashMap<RoomId, RoomRecord>,
    rooms_tx: HashMap<ClientId, mpsc::Sender<RoomMessage>>,

    next_client_id: ClientId,
    next_room_id: RoomId,
    chats: Vec<ChatRecord>,
}

impl LobbyActor {
    pub fn new(rx: mpsc::Receiver<LobbyMessage>) -> Self {
        Self {
            rx,
            clients: HashMap::new(),
            clients_tx: HashMap::new(),
            rooms: HashMap::new(),
            rooms_tx: HashMap::new(),
            next_client_id: 0,
            next_room_id: 0,
            chats: vec![],
        }
    }

    async fn send_to_client(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(client_tx) = self.clients_tx.get(&client_id) {
            client_tx.send(msg).await.unwrap();
        } else {
            error!("client[{client_id}] not exist");
        }
    }

    async fn send_to_room(&self, room_id: RoomId, msg: RoomMessage) {
        if let Some(room_tx) = self.rooms_tx.get(&room_id) {
            room_tx.send(msg).await.unwrap();
        } else {
            error!("room[{room_id}] not exist");
        }
    }

    async fn broadcast(&self, msg: DownlinkMessage) {
        for (client_id, client_record) in &self.clients {
            match client_record.location {
                ClientLocation::AtLobby => {
                    self.send_to_client(*client_id, msg.clone()).await;
                }
                ClientLocation::AtRoom(_) => {}
            }
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
                let msg = DownlinkMessage::LobbyUpdate {
                    info: LobbyPartialInfo::Chat(ChatRecord { client_id, content }),
                };

                self.broadcast(msg).await;
            }
            UplinkMessage::LobbyCreateRoom {
                client_id,
                req_id,
                room_name,
            } => {
                let Some(client) = self.clients.get_mut(&client_id) else {
                    error!("client[{client_id}] not exist");
                    return;
                };

                let (room_tx, room_rx) = mpsc::channel(32);
                let room_actor = RoomActor::new(room_rx, room_name.clone(), client_id);
                tokio::spawn(room_actor.run());

                let room_id = self.next_room_id;
                let room_record = RoomRecord { room_name };
                self.rooms.insert(room_id, room_record.clone());
                self.rooms_tx.insert(room_id, room_tx);

                client.location = ClientLocation::AtRoom(room_id);
                self.send_to_client(
                    client_id,
                    DownlinkMessage::LobbyCreateRoomAck {
                        req_id,
                        room_id: Some(room_id),
                    },
                )
                .await;

                self.broadcast(DownlinkMessage::LobbyUpdate {
                    info: LobbyPartialInfo::Room {
                        room_id,
                        room_record: room_record.clone(),
                    },
                })
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
                        ClientRecord {
                            location: ClientLocation::AtLobby,
                        },
                    );
                    self.clients_tx.insert(client_id, client_tx);

                    client_id_tx.send(client_id).unwrap();
                    self.send_to_client(
                        client_id,
                        DownlinkMessage::Greeting {
                            client_id,
                            chats: self.chats.clone(),
                            rooms: self.rooms.clone(),
                        },
                    )
                    .await;

                    self.next_client_id += 1;
                }
                LobbyMessage::UnregisterClient { client_id } => {
                    let Some(client_info) = self.clients.remove(&client_id) else {
                        error!("client[{}] not exists", client_id);
                        continue;
                    };
                    self.clients_tx.remove(&client_id);

                    match client_info.location {
                        ClientLocation::AtLobby => {
                            info!("client[{}] leaves", client_id);
                        }
                        ClientLocation::AtRoom(room_id) => {
                            self.send_to_room(room_id, RoomMessage::Quit(client_id))
                                .await;
                        }
                    }
                }
                LobbyMessage::ClientMessage { msg } => self.handle_msg(msg).await,
                LobbyMessage::InfoUpdate { info } => todo!(),
            }
        }
    }
}
