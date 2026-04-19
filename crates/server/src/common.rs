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

#[derive(Clone, Serialize, Deserialize)]
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

pub enum UplinkMessage {
    Ping(ClientId), // Ping then wait for PingEcho, to calculate latency
    Quit(ClientId), // client shutdown

    LobbyEnter(ClientId),
    LobbyChat(String),

    RoomCreate(ClientId),
    RoomEnter(ClientId),
    RoomChat(String),
    // LobbyCreateTeam(ClientId),
    RoomQuit(ClientId),
}
