use std::collections::HashMap;

use log::error;
use tokio::sync::mpsc;

use crate::{
    common::{
        ClientId, DownlinkMessage, DownlinkMessageValue, RoomId, SessionId, UplinkLobbyMessage,
        UplinkMessage, UplinkMessageValue,
    },
    lobby::LobbyMessage,
    room::RoomMessage,
    session::SessionActorTx,
};

#[derive(Debug)]
pub enum RouterMessage {
    RegisterSession {
        session_id: SessionId,
        session_tx: SessionActorTx,
    },
    UnregisterSession {
        session_id: SessionId,
    },
    ClientMessage {
        session_id: SessionId,
        msg: UplinkMessage,
    },
}

pub struct RouterActor {
    rx: mpsc::Receiver<RouterMessage>, // [SessionActor, ...] -> RouterActor
    lobby_tx: mpsc::Sender<LobbyMessage>, // RouterActor -> LobbyActor
    rooms_tx: HashMap<RoomId, mpsc::Sender<RoomMessage>>, // RouterActor -> [RoomActor, ...]
    sessions_tx: HashMap<SessionId, SessionActorTx>, // RouterActor -> [SessionActor, ...]
    next_client_id: ClientId,
}

impl RouterActor {
    pub fn new(rx: mpsc::Receiver<RouterMessage>, lobby_tx: mpsc::Sender<LobbyMessage>) -> Self {
        Self {
            rx,
            lobby_tx,
            rooms_tx: HashMap::new(),
            sessions_tx: HashMap::new(),
            next_client_id: 0,
        }
    }

    async fn send_to_session(&self, session_id: SessionId, msg: DownlinkMessage) {
        if let Some(tx) = self.sessions_tx.get(&session_id) {
            tx.send(msg).await.unwrap();
        } else {
            error!("session[{session_id}] not exist");
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
                    session_id,
                    session_tx,
                } => {
                    self.sessions_tx.insert(session_id, session_tx);

                    self.send_to_session(
                        session_id,
                        DownlinkMessage {
                            req_id: 0,
                            msg: DownlinkMessageValue::Greeting(self.next_client_id),
                        },
                    )
                    .await;

                    self.next_client_id += 1;
                }
                RouterMessage::UnregisterSession { session_id } => {
                    self.sessions_tx.remove(&session_id);
                }
                RouterMessage::ClientMessage { session_id, msg } => {
                    let client_id = msg.client_id;
                    let req_id = msg.req_id;
                    if let Some(tx) = self.sessions_tx.get(&client_id) {
                        match msg.msg {
                            UplinkMessageValue::Ping => todo!(),
                            UplinkMessageValue::Quit => todo!(),
                            UplinkMessageValue::Lobby(lobby_message) => match lobby_message {
                                UplinkLobbyMessage::Enter => {
                                    self.send_to_lobby(LobbyMessage::Enter {
                                        client_id,
                                        req_id,
                                        tx: tx.clone(),
                                    })
                                    .await;
                                }
                                UplinkLobbyMessage::Chat { content } => {
                                    self.send_to_lobby(LobbyMessage::Chat(client_id, content))
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
