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

macro_rules! check_client_in_lobby {
    ($self:expr, $client_id:expr) => {{
        let Some(client_record) = $self.clients.get_mut(&$client_id) else {
            error!("client[{}] not exist", $client_id);
            return;
        };

        if !matches!(client_record.location, ClientLocation::AtLobby) {
            error!("client[{}] @ {:?}", $client_id, client_record.location);
            return;
        }

        client_record
    }};
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
            if matches!(client_record.location, ClientLocation::AtLobby) {
                self.send_to_client(*client_id, msg.clone()).await;
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
                check_client_in_lobby!(self, client_id);

                info!("client[{client_id}] says '{content}'");
                self.chats.push(ChatRecord {
                    client_id,
                    content: content.clone(),
                });
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
                let client_record = check_client_in_lobby!(self, client_id);

                let client_tx = self.clients_tx.get(&client_id).unwrap();
                let (room_tx, room_rx) = mpsc::channel(32);
                tokio::spawn(
                    RoomActor::new(room_rx, room_name.clone(), client_id, client_tx.clone()).run(),
                );

                let room_id = self.next_room_id;
                self.next_room_id += 1;
                let room_record = RoomRecord { room_name };
                self.rooms.insert(room_id, room_record.clone());
                self.rooms_tx.insert(room_id, room_tx);

                client_record.location = ClientLocation::AtRoom(room_id);
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
                    self.next_client_id += 1;

                    // safe check
                    if self.clients.contains_key(&client_id) {
                        error!("client[{}] already exists", client_id);
                        continue;
                    }
                    assert!(!self.clients_tx.contains_key(&client_id));

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
                }
                LobbyMessage::UnregisterClient { client_id } => {
                    let Some(client_record) = self.clients.remove(&client_id) else {
                        error!("client[{}] not exists", client_id);
                        continue;
                    };
                    assert!(self.clients_tx.contains_key(&client_id));
                    self.clients_tx.remove(&client_id);

                    match client_record.location {
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
