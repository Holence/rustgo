use rustgo::{Coord, Stone};
use serde::{Deserialize, Serialize};

pub type SessionId = u64;
pub type ClientId = u64;
pub type RoomId = u64;

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Move { stone: Stone, coord: Coord },
    Pass,
    Resign,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DownlinkMessage {
    ServerGreeting(ClientId), // give client a unique ID
    ServerPingEcho,           // response to client Ping
    ServerShutdown,           // server shutdown

    LobbyInfo(Vec<RoomId>),
    LobbyChat(ClientId, String),

    RoomCreateAck(RoomId),
    // RoomInfo(Vec<TeamInfo>), // TODO full room info
    RoomChat(ClientId, String),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UplinkLobbyMessage {
    Enter,
    Chat { content: String },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UplinkRoomMessage {
    Create,
    Enter { room_id: RoomId },
    Chat { room_id: RoomId, content: String },
    Quit { room_id: RoomId },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UplinkMessageValue {
    Ping,
    Quit,
    Lobby(UplinkLobbyMessage),
    Room(UplinkRoomMessage),
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UplinkMessage {
    pub client_id: ClientId,
    pub msg: UplinkMessageValue,
}
