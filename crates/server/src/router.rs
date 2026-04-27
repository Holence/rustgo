use std::collections::HashMap;

use log::error;
use tokio::sync::{mpsc, oneshot};

use crate::{
    common::{ClientId, DownlinkMessage, RoomId, UplinkMessage},
    lobby::LobbyMessage,
    room::RoomMessage,
    session::SessionActorTx,
};

// TODO 真的有必要加这么一层router吗，直接由Lobby做转发不是更省事吗，这里还得额外维护记录rooms、sessions
#[derive(Debug)]
pub enum RouterMessage {
    RegisterSession {
        client_id_tx: oneshot::Sender<ClientId>,
        session_tx: SessionActorTx,
    },
    UnregisterSession {
        client_id: ClientId,
    },
    ClientMessage {
        msg: UplinkMessage,
    },
    RegisterRoom {
        room_id: RoomId,
        room_tx: mpsc::Sender<RoomMessage>,
    },
    UnregisterRoom {
        room_id: RoomId,
    },
}

enum ClientLocation {
    AtLobby,
    AtRoom(RoomId),
}
pub struct RouterActor {
    rx: mpsc::Receiver<RouterMessage>, // [SessionActor, ...] -> RouterActor
    lobby_tx: mpsc::Sender<LobbyMessage>, // RouterActor -> LobbyActor
    rooms_tx: HashMap<RoomId, mpsc::Sender<RoomMessage>>, // RouterActor -> [RoomActor, ...]
    sessions_tx: HashMap<ClientId, SessionActorTx>, // RouterActor -> [SessionActor, ...]
    clients_location: HashMap<ClientId, ClientLocation>,
    next_client_id: ClientId,
}

impl RouterActor {
    pub fn new(rx: mpsc::Receiver<RouterMessage>, lobby_tx: mpsc::Sender<LobbyMessage>) -> Self {
        Self {
            rx,
            lobby_tx,
            rooms_tx: HashMap::new(),
            sessions_tx: HashMap::new(),
            clients_location: HashMap::new(),
            next_client_id: 0,
        }
    }

    async fn send_to_session(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(tx) = self.sessions_tx.get(&client_id) {
            tx.send(msg).await.unwrap();
        } else {
            error!("session[{client_id}] not exist");
        }
    }

    async fn send_to_lobby(&self, msg: LobbyMessage) {
        self.lobby_tx.send(msg).await.unwrap();
    }

    async fn send_to_room(&self, room_id: RoomId, msg: RoomMessage) {
        if let Some(tx) = self.rooms_tx.get(&room_id) {
            tx.send(msg).await.unwrap();
        } else {
            error!("room[{room_id}] not exist");
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                RouterMessage::RegisterSession {
                    client_id_tx,
                    session_tx,
                } => {
                    let client_id = self.next_client_id;
                    self.sessions_tx.insert(client_id, session_tx);

                    client_id_tx.send(client_id).unwrap();
                    self.send_to_session(client_id, DownlinkMessage::Greeting { client_id })
                        .await;

                    self.next_client_id += 1;
                }
                RouterMessage::UnregisterSession { client_id } => {
                    self.sessions_tx.remove(&client_id);
                    if let Some(location) = self.clients_location.get(&client_id) {
                        match location {
                            ClientLocation::AtLobby => {
                                self.send_to_lobby(LobbyMessage::Quit { client_id }).await;
                            }
                            ClientLocation::AtRoom(_) => todo!(),
                        }
                    }
                }
                RouterMessage::ClientMessage { msg } => match msg {
                    UplinkMessage::Ping { client_id, req_id } => todo!(),
                    UplinkMessage::Quit { client_id } => todo!(),
                    UplinkMessage::LobbyEnter { client_id, req_id } => {
                        if let Some(location) = self.clients_location.get(&client_id) {
                            match location {
                                ClientLocation::AtLobby => {
                                    error!("client[{}] already at lobby", client_id);
                                }
                                ClientLocation::AtRoom(_) => todo!(),
                            }
                        } else {
                            self.send_to_lobby(LobbyMessage::Enter {
                                client_id,
                                req_id,
                                tx: self.sessions_tx.get(&client_id).unwrap().clone(),
                            })
                            .await;
                            self.clients_location
                                .insert(client_id, ClientLocation::AtLobby);
                        }
                    }
                    UplinkMessage::LobbyChat { client_id, content } => {
                        self.send_to_lobby(LobbyMessage::Chat { client_id, content })
                            .await;
                    }
                    UplinkMessage::LobbyCreateRoom {
                        client_id,
                        req_id,
                        room_name,
                    } => {
                        self.send_to_lobby(LobbyMessage::CreateRoom {
                            client_id,
                            req_id,
                            room_name,
                        })
                        .await
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
                },
                RouterMessage::RegisterRoom { room_id, room_tx } => {
                    if self.rooms_tx.contains_key(&room_id) {
                        error!("room[{room_id}] already exsist");
                        continue;
                    }
                    self.rooms_tx.insert(room_id, room_tx);
                }
                RouterMessage::UnregisterRoom { room_id } => todo!(),
            }
        }
    }
}
