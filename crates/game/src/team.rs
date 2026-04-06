use std::collections::HashMap;

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
    team_id: TeamId,
    stone: Stone,
    players: Vec<PlayerHandle>,
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

pub struct TeamBuilder {
    player_team_map: HashMap<PlayerId, TeamId>,
    team_infos: HashMap<TeamId, TeamInfo>,
}
impl TeamBuilder {
    pub fn new() -> Self {
        TeamBuilder {
            player_team_map: HashMap::new(),
            team_infos: HashMap::new(),
        }
    }

    pub fn add_team(&mut self, team_id: TeamId, stone: Stone) -> Result<(), ()> {
        if self.team_infos.contains_key(&team_id) {
            return Err(());
        }

        self.team_infos.insert(
            team_id,
            TeamInfo {
                team_id,
                stone,
                players: vec![],
            },
        );
        Ok(())
    }

    pub fn add_player(
        &mut self,
        team_id: TeamId,
        player_id: PlayerId,
        player_name: String,
    ) -> Result<(), ()> {
        if self.player_team_map.contains_key(&player_id) {
            return Err(());
        }
        let team = self.team_infos.get_mut(&team_id).ok_or(())?;
        team.players.push(PlayerInfo {
            player_id,
            team_id,
            player_name,
            eaten_stones: 0,
            time_left: 0,
        });

        Ok(())
    }

    pub fn take(self) -> Vec<TeamInfo> {
        let mut vec: Vec<TeamInfo> = self.team_infos.into_values().collect();
        vec.sort_unstable_by_key(|t| t.team_id.0);
        return vec;
    }
}
