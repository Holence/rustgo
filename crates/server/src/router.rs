use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::{
    common::{RoomId, SessionId, UplinkMessage},
    lobby::LobbyMessage,
    room::RoomMessage,
    session::SessionActorTx,
};

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
    sessions: HashMap<SessionId, SessionActorTx>, // RouterActor -> [SessionActor, ...]
    rooms: HashMap<RoomId, mpsc::Sender<RoomMessage>>, // RouterActor -> [RoomActor, ...]
    next_room_id: RoomId,
}

impl RouterActor {
    pub fn new(rx: mpsc::Receiver<RouterMessage>, lobby_tx: mpsc::Sender<LobbyMessage>) -> Self {
        Self {
            rx,
            lobby_tx,
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            next_room_id: 0,
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                RouterMessage::RegisterSession {
                    session_id,
                    session_tx,
                } => {
                    self.sessions.insert(session_id, session_tx);
                }
                RouterMessage::UnregisterSession { session_id } => {
                    self.sessions.remove(&session_id);
                }
                RouterMessage::ClientMessage { session_id, msg } => match msg {
                    UplinkMessage::Ping(_) => todo!(),
                    UplinkMessage::Quit(_) => todo!(),
                    UplinkMessage::LobbyEnter(_) => todo!(),
                    UplinkMessage::LobbyChat(_) => todo!(),
                    UplinkMessage::RoomCreate(_) => todo!(),
                    UplinkMessage::RoomEnter(_) => todo!(),
                    UplinkMessage::RoomChat(_) => todo!(),
                    UplinkMessage::RoomQuit(_) => todo!(),
                },
            }
        }
    }
}
