use std::collections::HashMap;

use log::error;
use tokio::sync::{mpsc, oneshot};

use crate::{
    common::{
        ClientId, DownlinkMessage, DownlinkMessageValue, RoomId, UplinkLobbyMessage, UplinkMessage,
        UplinkMessageValue,
    },
    lobby::LobbyMessage,
    room::RoomMessage,
    session::SessionActorTx,
};

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
                    self.send_to_session(
                        client_id,
                        DownlinkMessage {
                            req_id: 0,
                            msg: DownlinkMessageValue::Greeting(client_id),
                        },
                    )
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
                RouterMessage::ClientMessage { msg } => {
                    let client_id = msg.client_id;
                    let req_id = msg.req_id;
                    if let Some(tx) = self.sessions_tx.get(&client_id) {
                        match msg.msg {
                            UplinkMessageValue::Ping => todo!(),
                            UplinkMessageValue::Quit => todo!(),
                            UplinkMessageValue::Lobby(lobby_message) => match lobby_message {
                                UplinkLobbyMessage::Enter => {
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
                                            tx: tx.clone(),
                                        })
                                        .await;
                                        self.clients_location
                                            .insert(client_id, ClientLocation::AtLobby);
                                    }
                                }
                                UplinkLobbyMessage::Chat { content } => {
                                    self.send_to_lobby(LobbyMessage::Chat { client_id, content })
                                        .await;
                                }
                                UplinkLobbyMessage::CreateRoom => todo!(),
                            },
                            UplinkMessageValue::Room(room_message) => todo!(),
                        }
                    } else {
                        error!("client_id {} not exist", client_id);
                    }
                }
            }
        }
    }
}
