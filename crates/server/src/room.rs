use tokio::sync::mpsc;

use crate::common::{ClientId, DownlinkMessage};

#[derive(Clone)]
pub enum RoomMessage {
    // TODO RoomInfo(RoomId, Vec<TeamInfo>) // downlink only
    Enter(ClientId, mpsc::Sender<DownlinkMessage>),
    RoomChat(ClientId, String),
    // TODO CreateTeam(PlayerId)
    // TODO JoinTeam(PlayerId, TeamId)
    // TODO LeaveTeam(PlayerId, TeamId)
    Quit(ClientId),
}
