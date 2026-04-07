use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};

use rustgo::{Coord, Stone};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::sleep,
};

use crate::{
    Action, PlayerMessage, ServerMessage,
    player::{PlayerError, PlayerId, PlayerTrait},
};

pub struct LocalGnugoPlayer {
    size: usize,
    child: Child,
    writer: ChildStdin,
    reader: BufReader<ChildStdout>,
}

impl Drop for LocalGnugoPlayer {
    fn drop(&mut self) {
        self.child.kill().unwrap();
        let code = self.child.wait().unwrap();
        dbg!(code);
    }
}

impl LocalGnugoPlayer {
    pub fn new(size: usize) -> io::Result<LocalGnugoPlayer> {
        let child = Command::new("gnugo")
            .args(["--mode=gtp", "--chinese-rules", "--level=10"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();
        let mut child = match child {
            Ok(child) => child,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    eprintln!("Error: 'gnugo' executable not found.");
                    eprintln!("Please install GNU Go and ensure it is in your PATH.");
                    eprintln!("Example (Debian/Ubuntu): sudo apt install gnugo");
                } else {
                    eprintln!("Failed to start gnugo: {}", e);
                }
                return io::Result::Err(e);
            }
        };

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let stdin = child.stdin.take().unwrap();

        let mut player = LocalGnugoPlayer {
            size: size,
            child: child,
            writer: stdin,
            reader: reader,
        };

        let resp = player.send_and_get_response(&format!("boardsize {size}\n"))?;
        if resp.trim() != "=" {
            return Err(io::Error::new(io::ErrorKind::Unsupported, ""));
        }

        let resp = player.send_and_get_response("protocol_version\n")?;
        if resp.trim() == "= 2" {
            return Ok(player);
        } else {
            return Err(io::Error::new(io::ErrorKind::Unsupported, ""));
        }
    }

    fn send_and_get_response(&mut self, s: &str) -> io::Result<String> {
        // TODO use async
        println!("send: {s:?}");
        self.writer.write_all(s.as_bytes())?;
        self.writer.flush()?;

        let mut response = String::new();
        loop {
            let mut line = String::new();
            let size = self.reader.read_line(&mut line)?;
            if size == 0 {
                // EOF
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, ""));
            }

            if line.as_str() == "\n" {
                // response end with a empty newline
                break;
            }

            response.push_str(&line);
        }
        println!("get_response: {response:?}");
        Ok(response)
    }

    async fn play(&mut self, stone: Stone, coord: Coord) -> Result<(), PlayerError> {
        let coord = coord.to_a1(self.size).unwrap();
        let s = if stone == Stone::BLACK {
            format!("play B {coord}\n").to_string()
        } else {
            format!("play W {coord}\n").to_string()
        };
        let resp = self.send_and_get_response(&s)?;

        if !resp.starts_with('=') {
            return Err(PlayerError::EngineError(resp));
        }
        Ok(())
    }

    async fn genmove(&mut self, stone: Stone) -> Result<Action, PlayerError> {
        let s = match stone {
            Stone::BLACK => "reg_genmove B\n".to_string(),
            Stone::WHITE => "reg_genmove W\n".to_string(),
            _ => todo!(),
        };
        let resp = self.send_and_get_response(&s)?;

        if resp.starts_with('=') {
            let s = &resp.trim()[2..];
            // TODO pass and resign
            Ok(Action::Move {
                stone,
                coord: Coord::from_a1(s, self.size).unwrap(),
            })
        } else {
            Err(PlayerError::EngineError(resp))
        }
    }
}

impl PlayerTrait for LocalGnugoPlayer {
    fn run(
        mut self,
        player_id: PlayerId,
        uplink_tx: Sender<PlayerMessage>,
        mut downlink_rx: Receiver<ServerMessage>,
    ) {
        tokio::spawn(async move {
            loop {
                if let Some(msg) = downlink_rx.recv().await {
                    match msg {
                        ServerMessage::PlayerMove { stone, coord, .. } => {
                            self.play(stone, coord).await.unwrap();
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
                            sleep(Duration::from_secs(1)).await;
                            let action = self.genmove(stone).await.unwrap();

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
                            panic!()
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use rustgo::board::Board;

    use super::*;

    #[tokio::test]
    async fn two_gnugo() {
        const BOARD_SIZE: usize = 9;
        let mut board = Board::new(BOARD_SIZE);
        let mut player1 = LocalGnugoPlayer::new(BOARD_SIZE).unwrap();
        let mut player2 = LocalGnugoPlayer::new(BOARD_SIZE).unwrap();

        let mut stone = Stone::BLACK;
        loop {
            if stone == Stone::BLACK {
                let action = player1.genmove(stone).await.unwrap();
                match action {
                    Action::Move { stone, coord } => {
                        board.place_stone(coord, stone).unwrap();
                        player1.play(stone, coord).await.unwrap();
                        player2.play(stone, coord).await.unwrap();
                    }
                    Action::Pass => todo!(),
                    Action::Resign => todo!(),
                }
            } else {
                let action = player2.genmove(stone).await.unwrap();
                match action {
                    Action::Move { stone, coord } => {
                        board.place_stone(coord, stone).unwrap();
                        player1.play(stone, coord).await.unwrap();
                        player2.play(stone, coord).await.unwrap();
                    }
                    Action::Pass => todo!(),
                    Action::Resign => todo!(),
                }
            }
            stone = stone.next_stone(2);
            println!("{}", board.board_string());
        }
    }
}
