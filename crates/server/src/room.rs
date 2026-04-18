use crate::common::ClientId;

#[derive(Clone)]
pub enum RoomMessage {
    // TODO RoomInfo(RoomId, Vec<TeamInfo>) // downlink only
    Enter(ClientId),
    RoomChat(ClientId, String),
    // TODO CreateTeam(PlayerId)
    // TODO JoinTeam(PlayerId, TeamId)
    // TODO LeaveTeam(PlayerId, TeamId)
    Quit(ClientId),
}
