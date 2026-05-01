use std::collections::HashMap;

use rustgo::{Coord, Stone};
use serde::{Deserialize, Serialize};

use crate::lobby::{ChatRecord, LobbyPartialInfo, RoomRecord};

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
        chats: Vec<ChatRecord>,
        rooms: HashMap<RoomId, RoomRecord>,
    },
    PingEcho,
    Shutdown,

    LobbyUpdate {
        info: LobbyPartialInfo,
    },

    LobbyCreateRoomAck {
        req_id: ReqId,
        room_id: Option<RoomId>,
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
    // TODO Login {
    //     client_id: String,
    // },
    Ping {
        client_id: ClientId,
        req_id: ReqId,
    },
    Quit {
        client_id: ClientId,
    },
    LobbyChat {
        client_id: ClientId,
        content: String,
    },
    LobbyCreateRoom {
        client_id: ClientId,
        req_id: ReqId,
        room_name: String,
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
