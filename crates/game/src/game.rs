use rustgo::{Stone, board::Board};

use crate::player::{MoveAction, PlayerError, PlayerTrait};

pub struct Team {
    stone: Stone,
    players: Vec<Box<dyn PlayerTrait>>,
}

impl Team {
    pub fn new(stone: Stone, players: Vec<Box<dyn PlayerTrait>>) -> Self {
        Self { stone, players }
    }

    pub fn play(
        &mut self,
        move_action: MoveAction,
        except_player_index: Option<usize>,
    ) -> Result<(), PlayerError> {
        let len = self.players.len();
        let except = match except_player_index {
            Some(except) => {
                assert!(except < len);
                except
            }
            None => len,
        };

        for i in 0..len {
            if i != except {
                self.players[i].play(move_action)?
            }
        }
        return Ok(());
    }

    pub fn genmove(&mut self, player_index: usize) -> Result<MoveAction, PlayerError> {
        let p = &mut self.players[player_index];
        p.genmove(self.stone)
    }
}

pub struct Game {
    board: Board,
    teams: Vec<Team>,
    cur_team_index: usize,
    cur_player_index: Vec<usize>,
}

impl Game {
    pub fn new(size: usize, teams: Vec<Team>) -> Self {
        // TODO check stone
        let len = teams.len();
        Self {
            board: Board::new(size),
            teams: teams,
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

    pub fn run(&mut self) {
        loop {
            let cur_team_index = self.cur_team_index;
            let cur_team = &mut self.teams[cur_team_index];
            let cur_player = self.cur_player_index[cur_team_index];
            let move_action = cur_team.genmove(cur_player).unwrap(); // TODO handle error

            if let MoveAction::Move { stone, coord } = move_action {
                assert!(stone == cur_team.stone);
                self.board.place_stone(coord, stone).expect("一致性校验"); // TODO handle error
                println!("{}", self.board.board_string());
            }
            cur_team.play(move_action, Some(cur_player)).unwrap(); // TODO handle error

            for i in 0..self.teams.len() {
                if i != cur_team_index {
                    self.teams[i].play(move_action, None).unwrap(); // TODO handle error
                }
            }

            // advance player index for cur_team
            let new_player_index = self.cur_player_index[cur_team_index] + 1;
            if new_player_index == self.teams[cur_team_index].players.len() {
                self.cur_player_index[cur_team_index] = 0;
            } else {
                self.cur_player_index[cur_team_index] = new_player_index;
            }

            // advance cur_team_index
            self.cur_team_index += 1;
            if self.cur_team_index == self.teams.len() {
                self.cur_team_index = 0;
            }
        }
    }
}
