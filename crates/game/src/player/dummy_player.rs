use std::time::Duration;

use rand::{RngExt, rngs::StdRng};
use rustgo::{Coord, Stone, board::Board};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};

use crate::{
    Action, PlayerMessage, ServerMessage,
    player::{PlayerError, PlayerHandle, PlayerId, PlayerTrait},
};

pub struct DummyPlayer {
    board: Board,
    rng: StdRng,
}

impl DummyPlayer {
    pub fn new(size: usize) -> Self {
        DummyPlayer {
            board: Board::new(size),
            rng: rand::make_rng(),
        }
    }

    pub fn random_coord(&mut self) -> Coord {
        // TODO random of usize???
        let idx = (self.rng.random::<u32>() % (self.board.size_square() as u32)) as usize;
        return self.board.coord(idx);
    }

    async fn run(
        mut self,
        player_id: PlayerId,
        uplink_tx: Sender<PlayerMessage>,
        mut downlink_rx: Receiver<ServerMessage>,
    ) {
        loop {
            if let Some(msg) = downlink_rx.recv().await {
                match msg {
                    ServerMessage::PlayerMove { stone, coord, .. } => {
                        self.play(stone, coord).unwrap();
                    }
                    ServerMessage::PlayerChat {
                        player_id: player_id2,
                        chat,
                    } => {
                        println!(
                            "Player[{}] hear Player[{}] says: {}",
                            player_id.0, player_id2.0, chat
                        );
                    }
                    ServerMessage::GenMove(stone) => {
                        sleep(Duration::from_micros(500)).await;
                        let action = self.genmove(stone).unwrap();

                        uplink_tx
                            .send(PlayerMessage::PlayerAction { player_id, action })
                            .await
                            .unwrap();

                        uplink_tx
                            .send(PlayerMessage::PlayerChat {
                                player_id,
                                chat: format!("i choose {:?}", action),
                            })
                            .await
                            .unwrap();
                    }
                    ServerMessage::Error(_) => {
                        uplink_tx
                            .send(PlayerMessage::PlayerChat {
                                player_id,
                                chat: "oh shit".to_string(),
                            })
                            .await
                            .unwrap();
                    }
                    _ => {}
                }
            }
        }
    }
}

impl PlayerTrait for DummyPlayer {
    // TODO 重复的代码
    fn spawn(self, player_id: PlayerId, uplink_tx: Sender<PlayerMessage>) -> PlayerHandle {
        let (downlink_tx, downlink_rx) = mpsc::channel(32);

        tokio::spawn(self.run(player_id, uplink_tx, downlink_rx));

        PlayerHandle {
            player_id,
            downlink_tx,
        }
    }

    fn play(&mut self, stone: Stone, coord: Coord) -> Result<(), PlayerError> {
        self.board.place_stone(coord, stone).unwrap();
        Ok(())
    }

    fn genmove(&mut self, stone: Stone) -> Result<Action, PlayerError> {
        let coord = self.random_coord();
        return Ok(Action::Move { stone, coord });
    }
}
