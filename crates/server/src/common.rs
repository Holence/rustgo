use std::collections::HashMap;

use rustgo::{Coord, Stone};
use serde::{Deserialize, Serialize};

use crate::{
    lobby::{LobbyRoomRecord, LobbySnapshot},
    room::{RoomClientAction, RoomClientRecord, RoomSnapshot},
};

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
    pub username: String,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DownlinkMessage {
    /// trigger by `Login`
    LoginAck {
        client_id: Option<ClientId>,
    },
    /// trigger by `Ping`
    Pong,

    /// trigger by `LobbyEnter`
    LobbyEnterAck {
        req_id: ReqId,
        success: bool,
        lobby_snapshot: LobbySnapshot,
    },

    /// trigger by `LobbyChat`
    LobbyChatUpdate {
        chat_record: ChatRecord,
    },

    LobbyRoomUpdate {
        room_record: LobbyRoomRecord,
    },

    /// trigger by `LobbyCreateRoom`
    LobbyCreateRoomAck {
        req_id: ReqId,
        success: bool,
        room_snapshot: RoomSnapshot,
    },

    /// trigger by `RoomEnter`
    RoomEnterAck {
        req_id: ReqId,
        success: bool,
        room_snapshot: RoomSnapshot,
    },

    RoomClientUpdate {
        action: RoomClientAction,
        client_record: RoomClientRecord,
    },

    /// trigger by `RoomChat`
    RoomChatUpdate {
        room_id: RoomId,
        client_id: ClientId,
        username: String,
        content: String,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UplinkMessage {
    /// the first message after connection
    /// ClientActor respond `LoginAck`
    Login { username: String },

    /// ClientActor respond `Pong`
    Ping { client_id: ClientId, req_id: ReqId },

    /// if client in Void
    /// - Lobby respond `LobbyEnterAck`
    /// - Lobby mark client in Lobby
    LobbyEnter { client_id: ClientId, req_id: ReqId },

    /// if client in lobby, then
    /// - Lobby record chat
    /// - Lobby broadcast `LobbyChatUpdate`
    LobbyChat {
        client_id: ClientId,
        content: String,
    },

    /// if client in lobby, then
    /// - Lobby create Room with host=client
    /// - Lobby mark client in Room
    /// - Lobby respond `LobbyCreateRoomAck`
    /// - Lobby broadcast `LobbyRoomUpdate`
    LobbyCreateRoom {
        client_id: ClientId,
        req_id: ReqId,
        room_name: String,
    },

    /// if client in lobby && has room, then
    /// - Lobby mark client in Room
    /// - Lobby broadcast `LobbyRoomUpdate`
    /// - Room respond `RoomEnterAck`
    /// - Room broadcast `RoomClientUpdate`
    RoomEnter {
        client_id: ClientId,
        req_id: ReqId,
        room_id: RoomId,
    },

    /// if client in room && room.state==Teaming
    /// - Room broadcast `RoomClientUpdate`
    RoomChangeTeam {
        client_id: ClientId,
        req_id: ReqId,
        room_id: RoomId,
        team: Option<Stone>,
    },

    /// if client in room, then
    /// - Room record chat
    /// - Room broadcast `RoomChatUpdate`
    RoomChat {
        client_id: ClientId,
        room_id: RoomId,
        content: String,
    },

    /// if client in room, then
    /// - Lobby mark client in Void
    /// - Lobby broadcast `LobbyRoomUpdate`
    /// - Room broadcast `RoomClientUpdate`
    RoomQuit {
        client_id: ClientId,
        req_id: ReqId,
        room_id: RoomId,
    },
}
