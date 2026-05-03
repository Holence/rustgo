use std::collections::HashMap;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

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

macro_rules! check_has_client {
    ($self:expr, $client_id:expr) => {{
        let Some(client_record) = $self.clients.get_mut(&$client_id) else {
            error!("client[{}] not exist", $client_id);
            return;
        };
        let client_tx = $self.clients_tx.get(&$client_id).unwrap();

        (client_record, client_tx)
    }};
}

macro_rules! check_client_in_void {
    ($self:expr, $client_id:expr, $client_record:expr) => {{
        if !matches!($client_record.location, ClientLocation::Void) {
            error!("client[{}] @ {:?}", $client_id, $client_record.location);
            return;
        }
    }};
}

macro_rules! check_client_in_lobby {
    ($self:expr, $client_id:expr, $client_record:expr) => {{
        if !matches!($client_record.location, ClientLocation::AtLobby) {
            error!("client[{}] @ {:?}", $client_id, $client_record.location);
            return;
        }
    }};
}

macro_rules! check_has_room {
    ($self:expr, $room_id:expr) => {{
        let Some(room_record) = $self.rooms.get_mut(&$room_id) else {
            error!("room[{}] not exist", $room_id);
            return;
        };
        let room_tx = $self.rooms_tx.get(&$room_id).unwrap();

        (room_record, room_tx)
    }};
}

macro_rules! check_client_in_room {
    ($self:expr, $client_id:expr, $client_record:expr, $room_id:expr) => {{
        if !matches!(&$client_record.location, ClientLocation::AtRoom(_id) if *_id == $room_id)
        {
            error!("client[{}] @ {:?}", $client_id, &$client_record.location);
            return;
        }
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
                let client_tx = self.clients_tx.get(client_id).unwrap();
                client_tx.send(msg.clone()).await.unwrap();
            }
        }
    }

    async fn handle_msg(&mut self, msg: UplinkMessage) {
        match msg {
            UplinkMessage::Ping { client_id, req_id } => todo!(),
            UplinkMessage::LobbyEnter { client_id, req_id } => {
                let (client_record, client_tx) = check_has_client!(self, client_id);
                check_client_in_void!(self, client_id, client_record);

                client_record.location = ClientLocation::AtLobby;
                client_tx
                    .send(DownlinkMessage::LobbyEnterAck {
                        req_id,
                        success: true,
                        chats: self.chats.clone(),
                        rooms: self.rooms.clone(),
                    })
                    .await
                    .unwrap();
            }
            UplinkMessage::LobbyChat { client_id, content } => {
                let (client_record, _) = check_has_client!(self, client_id);
                check_client_in_lobby!(self, client_id, client_record);

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
                let (client_record, client_tx) = check_has_client!(self, client_id);
                check_client_in_lobby!(self, client_id, client_record);

                let (room_tx, room_rx) = mpsc::channel(32);
                let room_id = self.next_room_id;
                self.next_room_id += 1;
                tokio::spawn(
                    RoomActor::new(
                        room_rx,
                        room_id,
                        room_name.clone(),
                        client_id,
                        client_tx.clone(),
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

                client_record.location = ClientLocation::AtRoom(room_id);
                client_tx
                    .send(DownlinkMessage::LobbyCreateRoomAck {
                        req_id,
                        room_id: Some(room_id),
                    })
                    .await
                    .unwrap();

                self.broadcast(DownlinkMessage::LobbyRoomUpdate { room_record })
                    .await;
            }
            UplinkMessage::RoomEnter {
                client_id,
                req_id,
                room_id,
            } => {
                let (room_record, room_tx) = check_has_room!(self, room_id);
                let (client_record, client_tx) = check_has_client!(self, client_id);
                check_client_in_lobby!(self, client_id, client_record);
                client_record.location = ClientLocation::AtRoom(room_id);

                room_record.client_nums += 1;
                room_tx
                    .send(RoomMessage::Enter {
                        req_id,
                        client_id,
                        client_tx: client_tx.clone(),
                    })
                    .await
                    .unwrap();

                let msg = DownlinkMessage::LobbyRoomUpdate {
                    room_record: room_record.clone(),
                };
                self.broadcast(msg).await;
            }
            UplinkMessage::RoomChat {
                client_id,
                room_id,
                content,
            } => {
                let (room_record, room_tx) = check_has_room!(self, room_id);
                let (client_record, client_tx) = check_has_client!(self, client_id);
                check_client_in_room!(self, client_id, client_record, room_id);
                room_tx
                    .send(RoomMessage::RoomChat { client_id, content })
                    .await
                    .unwrap();
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

                    client_id_tx.send(client_id).unwrap();
                    client_tx
                        .send(DownlinkMessage::Greeting { client_id })
                        .await
                        .unwrap();

                    self.clients.insert(
                        client_id,
                        ClientRecord {
                            location: ClientLocation::Void,
                        },
                    );
                    self.clients_tx.insert(client_id, client_tx);
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
