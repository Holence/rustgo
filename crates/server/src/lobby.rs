use std::collections::HashMap;

use log::{error, info};
use tokio::sync::{mpsc, oneshot};

use crate::{
    common::{ClientId, DownlinkMessage, RoomId, UplinkMessage},
    room::{RoomActor, RoomMessage},
};

pub enum LobbyMessage {
    RegisterSession {
        client_id_tx: oneshot::Sender<ClientId>,
        session_tx: mpsc::Sender<DownlinkMessage>,
    },
    UnregisterSession {
        client_id: ClientId,
    },
    ClientMessage {
        msg: UplinkMessage,
    },
}

enum ClientLocation {
    AtLobby,
    AtRoom(RoomId),
}

pub struct LobbyActor {
    rx: mpsc::Receiver<LobbyMessage>,
    rooms_tx: HashMap<RoomId, mpsc::Sender<RoomMessage>>, // LobbyActor -> [RoomActor, ...]
    sessions_tx: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>, // // LobbyActor -> [SessionActor, ...]
    next_client_id: ClientId,
    next_room_id: RoomId,
    clients_location: HashMap<ClientId, ClientLocation>,
}

impl LobbyActor {
    pub fn new(rx: mpsc::Receiver<LobbyMessage>) -> Self {
        Self {
            rx,
            rooms_tx: HashMap::new(),
            sessions_tx: HashMap::new(),
            next_client_id: 0,
            next_room_id: 0,
            clients_location: HashMap::new(),
        }
    }

    async fn send_to_session(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(tx) = self.sessions_tx.get(&client_id) {
            tx.send(msg).await.unwrap();
        } else {
            error!("client {client_id} not exist");
        }
    }

    async fn handle_msg(&mut self, msg: UplinkMessage) {
        match msg {
            UplinkMessage::Ping { client_id, req_id } => todo!(),
            UplinkMessage::Quit { client_id } => {
                self.sessions_tx.remove(&client_id);
            }
            UplinkMessage::LobbyEnter { client_id, req_id } => {
                self.send_to_session(
                    client_id,
                    DownlinkMessage::LobbyEnterAck {
                        req_id,
                        success: true,
                    },
                )
                .await;
            }
            UplinkMessage::LobbyChat { client_id, content } => {
                if !self.sessions_tx.contains_key(&client_id) {
                    error!("session[{client_id}] not exist");
                    return;
                }

                info!("hear client[{client_id}] says '{content}'");
                let msg = DownlinkMessage::LobbyChat { client_id, content };
                for tx in self.sessions_tx.values() {
                    tx.send(msg.clone()).await.unwrap();
                }
            }
            UplinkMessage::LobbyCreateRoom {
                client_id,
                req_id,
                room_name,
            } => {
                if !self.sessions_tx.contains_key(&client_id) {
                    error!("session[{client_id}] not exist");
                    return;
                }

                let (room_tx, room_rx) = mpsc::channel(32);
                let room_actor = RoomActor::new(room_rx, room_name, client_id);
                tokio::spawn(room_actor.run());

                let room_id = self.next_room_id;
                self.rooms_tx.insert(room_id, room_tx.clone());

                self.send_to_session(
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
                LobbyMessage::RegisterSession {
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
                LobbyMessage::UnregisterSession { client_id } => {
                    if !self.sessions_tx.contains_key(&client_id) {
                        error!("session[{}] not exists", client_id);
                    }
                    self.sessions_tx.remove(&client_id);
                    if let Some(location) = self.clients_location.get(&client_id) {
                        match location {
                            ClientLocation::AtLobby => todo!(),
                            ClientLocation::AtRoom(_) => todo!(),
                        }
                    }
                }
                LobbyMessage::ClientMessage { msg } => self.handle_msg(msg).await,
            }
        }
    }
}
