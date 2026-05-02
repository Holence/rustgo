use std::collections::HashMap;

use rustgo::{Coord, Stone};
use serde::{Deserialize, Serialize};

use crate::lobby::{LobbyPartialInfo, RoomRecord};

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
pub struct ChatRecord {
    pub client_id: ClientId,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DownlinkMessage {
    /// trigger by `RegisterClient`
    Greeting {
        client_id: ClientId,
    },
    PingEcho,

    /// trigger by `LobbyEnter`
    LobbyEnterAck {
        req_id: ReqId,
        success: bool,
        chats: Vec<ChatRecord>,
        rooms: HashMap<RoomId, RoomRecord>,
    },

    // TODO split LobbyPartialInfo
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
        // TODO chats
        // TODO clients
    },
    RoomChat {
        room_id: RoomId,
        client_id: ClientId,
        content: String,
    },
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

    /// if client in Void
    /// - send `LobbyEnterAck`
    /// - mark client in Lobby
    LobbyEnter {
        client_id: ClientId,
        req_id: ReqId,
    },

    /// if client in lobby, then
    /// - record chat
    /// - broadcast `LobbyPartialInfo::Chat`
    LobbyChat {
        client_id: ClientId,
        content: String,
    },

    /// if client in lobby, then
    /// - create Room with host=client
    /// - mark client in Room
    /// - send `LobbyCreateRoomAck`
    /// - broadcast `LobbyPartialInfo::Room`
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
