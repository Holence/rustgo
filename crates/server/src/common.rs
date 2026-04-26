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
pub enum DownlinkMessage {
    Greeting {
        client_id: ClientId,
    },
    PingEcho,
    Shutdown,

    LobbyEnterAck {
        req_id: ReqId,
        success: bool,
    },
    LobbyChat {
        client_id: ClientId,
        content: String,
    },

    RoomCreateAck {
        req_id: ReqId,
        success: bool,
        room_id: RoomId,
    },
    RoomEnterAck {
        req_id: ReqId,
        success: bool,
        room_id: RoomId,
    },
    RoomChat {
        room_id: RoomId,
        client_id: ClientId,
        content: String,
    },
    RoomQuitAck,
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
