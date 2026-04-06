use rustgo::{Coord, Stone};

use crate::{
    player::{PlayerId, PlayerInfo},
    team::{TeamId, TeamInfo},
};

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Move { stone: Stone, coord: Coord },
    Pass,
    Resign,
}

#[derive(Clone, Debug)]
pub enum ServerMessage {
    GameStart(Vec<TeamInfo>),
    GameUpdate {
        cur_team: Option<TeamId>,
        cur_player: Option<PlayerId>,
        player_info: Option<Vec<PlayerInfo>>,
    },
    PlayerMove {
        player_id: PlayerId,
        stone: Stone,
        coord: Coord,
    },
    PlayerChat {
        player_id: PlayerId,
        chat: String,
    },
    GenMove(Stone),
    Error(String),
    GameOver,
}

#[derive(Clone, Debug)]
pub enum PlayerMessage {
    PlayerAction { player_id: PlayerId, action: Action },
    PlayerChat { player_id: PlayerId, chat: String },
}
