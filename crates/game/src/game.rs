use std::vec;

use rustgo::{Stone, board::Board};
use tokio::sync::mpsc::Receiver;

use crate::{
    Action, PlayerMessage, ServerMessage,
    player::{PlayerId, PlayerInfo},
    team::{TeamHandle, TeamId, TeamInfo},
};

pub struct Game {
    board: Board,

    uplink_rx: Receiver<PlayerMessage>,

    team_handles: Vec<TeamHandle>,
    team_infos: Vec<TeamInfo>,
    cur_team_index: usize,
    cur_player_index: Vec<usize>,
}

impl Game {
    pub fn new(
        size: usize,
        uplink_rx: Receiver<PlayerMessage>,
        team_infos: Vec<TeamInfo>,
        team_handles: Vec<TeamHandle>,
    ) -> Self {
        // TODO check stone
        let len = team_handles.len();
        Self {
            board: Board::new(size),
            uplink_rx: uplink_rx,
            team_handles: team_handles,
            team_infos: team_infos,
            cur_team_index: 0,
            cur_player_index: vec![0; len],
        }
    }

    pub fn size(&self) -> usize {
        self.board.size()
    }

    pub fn board(&self) -> &[Stone] {
        self.board.board_array()
    }

    async fn broadcast(&mut self, msg: ServerMessage) {
        for team in &mut self.team_handles {
            team.broadcast(msg.clone()).await;
        }
    }

    fn cur_stone(&self) -> Stone {
        self.team_handles[self.cur_team_index].stone()
    }
    fn cur_player_id(&self) -> PlayerId {
        self.team_handles[self.cur_team_index].player_id(self.cur_player_index[self.cur_team_index])
    }
    fn cur_team_id(&self) -> TeamId {
        self.team_handles[self.cur_team_index].team_id()
    }

    async fn send(&mut self, msg: ServerMessage) {
        let cur_team_index = self.cur_team_index;
        let cur_team = &mut self.team_handles[cur_team_index];
        cur_team
            .send(self.cur_player_index[cur_team_index], msg)
            .await;
    }

    async fn genmove(&mut self) {
        self.send(ServerMessage::GenMove(self.cur_stone())).await;
    }

    pub async fn run(&mut self) {
        // 广播开局信息
        self.broadcast(ServerMessage::GameStart(self.team_infos.clone()))
            .await;

        self.genmove().await;
        while let Some(msg) = self.uplink_rx.recv().await {
            dbg!(&msg);
            match msg {
                PlayerMessage::PlayerAction { player_id, action } => {
                    match action {
                        Action::Move { stone, coord } => {
                            assert!(player_id == self.cur_player_id());
                            assert!(stone == self.cur_stone());

                            let res = self.board.place_stone(coord, stone);
                            match res {
                                Ok(eaten) => {
                                    println!("{}", self.board.board_string());
                                    // 广播落子信息
                                    self.broadcast(ServerMessage::PlayerMove {
                                        player_id,
                                        stone,
                                        coord,
                                    })
                                    .await;

                                    let cur_team_index = self.cur_team_index;

                                    // advance player index for cur_team
                                    let new_player_index =
                                        self.cur_player_index[cur_team_index] + 1;
                                    if new_player_index
                                        == self.team_handles[cur_team_index].player_nums()
                                    {
                                        self.cur_player_index[cur_team_index] = 0;
                                    } else {
                                        self.cur_player_index[cur_team_index] = new_player_index;
                                    }

                                    // advance cur_team_index
                                    self.cur_team_index += 1;
                                    if self.cur_team_index == self.team_handles.len() {
                                        self.cur_team_index = 0;
                                    }

                                    self.broadcast(ServerMessage::GameUpdate {
                                        cur_team: Some(
                                            self.team_handles[self.cur_team_index].team_id(),
                                        ),
                                        cur_player: Some(self.cur_player_id()),
                                        player_info: None, // TODO 更新吃子、计时信息
                                    })
                                    .await;
                                }
                                Err(s) => {
                                    self.send(ServerMessage::Error(s.to_string())).await;
                                }
                            }
                        }
                        Action::Pass => todo!(),
                        Action::Resign => todo!(),
                    }
                    self.genmove().await;
                }
                PlayerMessage::PlayerChat { player_id, chat } => {
                    self.broadcast(ServerMessage::PlayerChat { player_id, chat })
                        .await;
                }
            }
        }
    }
}
