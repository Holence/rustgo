use std::collections::HashMap;

use log::{error, info};
use tokio::sync::mpsc;

use crate::{
    common::{ClientId, DownlinkMessage, ReqId, RoomId},
    room::{RoomActor, RoomMessage},
    router::RouterMessage,
};

#[derive(Clone)]
pub enum LobbyMessage {
    Enter {
        client_id: ClientId,
        req_id: ReqId,
        tx: mpsc::Sender<DownlinkMessage>,
    },
    Chat {
        client_id: ClientId,
        content: String,
    },
    CreateRoom {
        client_id: ClientId,
        req_id: ReqId,
        room_name: String,
    },
    Quit {
        client_id: ClientId,
    },
}

pub struct LobbyActor {
    rx: mpsc::Receiver<LobbyMessage>,
    router_tx: mpsc::Sender<RouterMessage>,
    rooms_tx: HashMap<RoomId, mpsc::Sender<RoomMessage>>, // RouterActor -> [RoomActor, ...]
    sessions_tx: HashMap<ClientId, mpsc::Sender<DownlinkMessage>>,
    next_room_id: RoomId,
}

impl LobbyActor {
    pub fn new(rx: mpsc::Receiver<LobbyMessage>, router_tx: mpsc::Sender<RouterMessage>) -> Self {
        Self {
            rx,
            router_tx,
            rooms_tx: HashMap::new(),
            sessions_tx: HashMap::new(),
            next_room_id: 0,
        }
    }

    async fn send_to_session(&self, client_id: ClientId, msg: DownlinkMessage) {
        if let Some(tx) = self.sessions_tx.get(&client_id) {
            tx.send(msg).await.unwrap();
        } else {
            error!("client {client_id} not exist");
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                LobbyMessage::Enter {
                    client_id,
                    req_id,
                    tx,
                } => {
                    self.sessions_tx.insert(client_id, tx);
                    self.send_to_session(
                        client_id,
                        DownlinkMessage::LobbyEnterAck {
                            req_id,
                            success: true,
                        },
                    )
                    .await;
                }
                LobbyMessage::Quit { client_id } => {
                    self.sessions_tx.remove(&client_id);
                }
                LobbyMessage::CreateRoom {
                    client_id,
                    req_id,
                    room_name,
                } => {
                    if !self.sessions_tx.contains_key(&client_id) {
                        error!("session[{client_id}] not exist");
                        continue;
                    }

                    let (room_tx, room_rx) = mpsc::channel(32);
                    let room_actor = RoomActor::new(room_rx, room_name, client_id);
                    tokio::spawn(room_actor.run());

                    let room_id = self.next_room_id;
                    self.rooms_tx.insert(room_id, room_tx.clone());

                    // notify router
                    self.router_tx
                        .send(RouterMessage::RegisterRoom { room_id, room_tx })
                        .await
                        .unwrap();

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
                LobbyMessage::Chat { client_id, content } => {
                    if !self.sessions_tx.contains_key(&client_id) {
                        error!("session[{client_id}] not exist");
                        continue;
                    }

                    info!("hear client[{client_id}] says '{content}'");
                    let msg = DownlinkMessage::LobbyChat { client_id, content };
                    for tx in self.sessions_tx.values() {
                        tx.send(msg.clone()).await.unwrap();
                    }
                }
            }
        }
    }
}
