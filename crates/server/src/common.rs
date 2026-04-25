use rustgo::{Coord, Stone};
use serde::{Deserialize, Serialize};

pub type ClientId = u64;
pub type RoomId = u64;
pub type ReqId = u64;

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Move { stone: Stone, coord: Coord },
    Pass,
    Resign,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DownlinkLobbyMessage {
    EnterAck {
        success: bool,
    },
    Chat {
        client_id: ClientId,
        content: String,
    },
    // Update{ room info }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DownlinkRoomMessage {
    CreateAck {
        success: bool,
        room_id: RoomId,
    },
    EnterAck {
        success: bool,
        room_id: RoomId,
    },
    Chat {
        room_id: RoomId,
        client_id: ClientId,
        content: String,
    },
    QuitAck,
    // Update { team info}
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DownlinkMessageValue {
    Greeting(ClientId),
    PingEcho,
    Shutdown,
    Lobby(DownlinkLobbyMessage),
    Room(DownlinkRoomMessage),
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DownlinkMessage {
    pub req_id: ReqId,
    pub msg: DownlinkMessageValue,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UplinkMessage {
    Ping {
        client_id: ClientId,
        req_id: ReqId,
    },
    Quit {
        client_id: ClientId,
    },
    LobbyEnter {
        client_id: ClientId,
        req_id: ReqId,
    },
    LobbyChat {
        client_id: ClientId,
        content: String,
    },
    LobbyCreateRoom {
        client_id: ClientId,
        req_id: ReqId,
    },
    RoomEnter {
        client_id: ClientId,
        req_id: ReqId,
        room_id: RoomId,
    },
    RoomChat {
        client_id: ClientId,
        room_id: RoomId,
        content: String,
    },
    RoomQuit {
        client_id: ClientId,
        req_id: ReqId,
        room_id: RoomId,
    },
}
