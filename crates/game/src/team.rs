use rustgo::Stone;

use crate::{
    ServerMessage,
    player::{PlayerHandle, PlayerId, PlayerInfo},
};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct TeamId(usize);
impl TeamId {
    pub fn new(id: usize) -> Self {
        TeamId(id)
    }
}

#[derive(Clone, Debug)]
pub struct TeamInfo {
    team_id: TeamId,
    stone: Stone,
    players: Vec<PlayerInfo>,
}

pub struct TeamHandle {
    pub team_id: TeamId,
    pub stone: Stone,
    pub players: Vec<PlayerHandle>,
}
impl TeamHandle {
    pub fn new(team_id: TeamId, stone: Stone, players: Vec<PlayerHandle>) -> Self {
        Self {
            team_id,
            stone,
            players,
        }
    }

    pub async fn send(&self, player_index: usize, msg: ServerMessage) {
        let player = &self.players[player_index];
        player.send(msg).await;
    }

    pub fn stone(&self) -> Stone {
        self.stone
    }

    pub fn player_nums(&self) -> usize {
        self.players.len()
    }

    pub fn player_id(&self, player_index: usize) -> PlayerId {
        self.players[player_index].player_id
    }

    pub fn team_id(&self) -> TeamId {
        self.team_id
    }

    pub async fn broadcast(&mut self, msg: ServerMessage) {
        for player in &mut self.players {
            player.send(msg.clone()).await;
        }
    }
}
