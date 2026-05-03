use std::collections::HashMap;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, mpsc::Sender, oneshot};

use crate::{
    common::{ChatRecord, ClientId, DownlinkMessage, RoomId, UplinkMessage},
    room::{RoomActor, RoomMessage},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
enum ClientLocation {
    Void,
    AtLobby,
    AtRoom(RoomId),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ClientRecord {
    location: ClientLocation,
    // name
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RoomRecord {
    pub room_id: RoomId,
    pub room_name: String,
    pub client_nums: usize,
    // room_state // state: Teaming/GameStart/GameEnd
}

pub enum LobbyMessage {
    /// send by client when connection established
    /// - sned `Greeting`
    /// - mark client in Void
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
    // TODO room update
    // RoomStateUpdate
    // RoomActionUpdate
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

    fn ensure_client_location(&self, client_id: ClientId, expected: ClientLocation) -> bool {
        let Some(client_record) = self.clients.get(&client_id) else {
            error!("client[{}] not exist", client_id);
            return false;
        };

        let ok = match (expected, &client_record.location) {
            (ClientLocation::Void, ClientLocation::Void) => true,
            (ClientLocation::AtLobby, ClientLocation::AtLobby) => true,
            (ClientLocation::AtRoom(expected_id), ClientLocation::AtRoom(actual_id)) => {
                expected_id == *actual_id
            }
            _ => false,
        };

        if !ok {
            error!("client[{}] @ {:?}", client_id, client_record.location);
        }

        ok
    }

    fn client_change_location(&mut self, client_id: ClientId, location: ClientLocation) {
        self.clients.get_mut(&client_id).unwrap().location = location;
    }

    fn client_tx_cloned(&self, client_id: ClientId) -> Sender<DownlinkMessage> {
        self.clients_tx.get(&client_id).unwrap().clone()
    }

    async fn send_to_client(&self, client_id: ClientId, msg: DownlinkMessage) {
        self.clients_tx
            .get(&client_id)
            .unwrap()
            .send(msg)
            .await
            .unwrap()
    }

    async fn send_to_room(&self, room_id: RoomId, msg: RoomMessage) {
        self.rooms_tx
            .get(&room_id)
            .unwrap()
            .send(msg)
            .await
            .unwrap()
    }

    async fn broadcast(&self, msg: DownlinkMessage) {
        for (client_id, client_record) in &self.clients {
            if matches!(client_record.location, ClientLocation::AtLobby) {
                let client_tx = self.clients_tx.get(client_id).unwrap();
                client_tx.send(msg.clone()).await.unwrap();
            }
        }
    }

    async fn handle_msg(&mut self, msg: UplinkMessage) {
        match msg {
            UplinkMessage::Ping { client_id, req_id } => todo!(),
            UplinkMessage::LobbyEnter { client_id, req_id } => {
                if !self.ensure_client_location(client_id, ClientLocation::Void) {
                    error!("");
                    return;
                }

                self.client_change_location(client_id, ClientLocation::AtLobby);
                self.send_to_client(
                    client_id,
                    DownlinkMessage::LobbyEnterAck {
                        req_id,
                        success: true,
                        chats: self.chats.clone(),
                        rooms: self.rooms.clone(),
                    },
                )
                .await;
            }
            UplinkMessage::LobbyChat { client_id, content } => {
                if !self.ensure_client_location(client_id, ClientLocation::AtLobby) {
                    error!("");
                    return;
                }

                info!("client[{client_id}] says '{content}'");
                let chat_record = ChatRecord { client_id, content };
                self.chats.push(chat_record.clone());
                self.broadcast(DownlinkMessage::LobbyChatUpdate { chat_record })
                    .await;
            }
            UplinkMessage::LobbyCreateRoom {
                client_id,
                req_id,
                room_name,
            } => {
                if !self.ensure_client_location(client_id, ClientLocation::AtLobby) {
                    error!("");
                    return;
                }

                let (room_tx, room_rx) = mpsc::channel(32);
                let room_id = self.next_room_id;
                self.next_room_id += 1;
                tokio::spawn(
                    RoomActor::new(
                        room_rx,
                        room_id,
                        room_name.clone(),
                        client_id,
                        self.client_tx_cloned(client_id),
                    )
                    .run(),
                );

                let room_record = RoomRecord {
                    room_id,
                    room_name,
                    client_nums: 1,
                };
                self.rooms.insert(room_id, room_record.clone());
                self.rooms_tx.insert(room_id, room_tx);

                self.client_change_location(client_id, ClientLocation::AtRoom(room_id));
                self.send_to_client(
                    client_id,
                    DownlinkMessage::LobbyCreateRoomAck {
                        req_id,
                        room_id: Some(room_id),
                    },
                )
                .await;
                self.broadcast(DownlinkMessage::LobbyRoomUpdate { room_record })
                    .await;
            }
            UplinkMessage::RoomEnter {
                client_id,
                req_id,
                room_id,
            } => {
                if !self.ensure_client_location(client_id, ClientLocation::AtLobby) {
                    error!("");
                    return;
                }

                let room_record = {
                    let Some(room_record) = self.rooms.get_mut(&room_id) else {
                        error!("room[{}] not exist", room_id);
                        return;
                    };
                    room_record.client_nums += 1;
                    room_record.clone()
                };

                self.client_change_location(client_id, ClientLocation::AtRoom(room_id));
                self.send_to_room(
                    room_id,
                    RoomMessage::Enter {
                        req_id,
                        client_id,
                        client_tx: self.client_tx_cloned(client_id),
                    },
                )
                .await;
                self.broadcast(DownlinkMessage::LobbyRoomUpdate { room_record })
                    .await;
            }
            UplinkMessage::RoomChat {
                client_id,
                room_id,
                content,
            } => {
                if !self.ensure_client_location(client_id, ClientLocation::AtRoom(room_id)) {
                    error!("");
                    return;
                }

                self.send_to_room(room_id, RoomMessage::RoomChat { client_id, content })
                    .await;
            }
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
                            location: ClientLocation::Void,
                        },
                    );
                    self.clients_tx.insert(client_id, client_tx.clone());

                    client_id_tx.send(client_id).unwrap();
                    client_tx
                        .send(DownlinkMessage::Greeting { client_id })
                        .await
                        .unwrap();
                }
                LobbyMessage::UnregisterClient { client_id } => {
                    let Some(client_record) = self.clients.remove(&client_id) else {
                        error!("client[{}] not exists", client_id);
                        continue;
                    };
                    assert!(self.clients_tx.contains_key(&client_id));
                    self.clients_tx.remove(&client_id);

                    match client_record.location {
                        ClientLocation::AtRoom(room_id) => {
                            self.send_to_room(room_id, RoomMessage::Quit(client_id))
                                .await;
                            // TODO DownlinkMessage::LobbyRoomUpdate
                        }
                        _ => {
                            info!("client[{}] leaves", client_id);
                        }
                    }
                }
                LobbyMessage::ClientMessage { msg } => self.handle_msg(msg).await,
            }
        }
    }
}
