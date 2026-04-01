use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use crate::{
    Coord, Stone,
    player::{MoveAction, PlayerError, PlayerTrait},
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
}

impl PlayerTrait for LocalGnugoPlayer {
    fn play(&mut self, move_action: MoveAction) -> Result<(), PlayerError> {
        match move_action {
            MoveAction::Move { stone, coord } => {
                let coord = coord.to_a1(self.size).unwrap();
                let s = if stone == Stone::BLACK {
                    format!("play B {coord}\n").to_string()
                } else {
                    format!("play W {coord}\n").to_string()
                };
                let resp = self.send_and_get_response(&s)?;

                if resp.starts_with('=') {
                    Ok(())
                } else {
                    Err(PlayerError::EngineError(resp))
                }
            }
            MoveAction::Pass => todo!(),
            MoveAction::Resign => todo!(),
        }
    }

    fn genmove(&mut self, stone: Stone) -> Result<MoveAction, PlayerError> {
        let s = if stone == Stone::BLACK {
            "genmove B\n".to_string()
        } else {
            "genmove W\n".to_string()
        };
        let resp = self.send_and_get_response(&s)?;

        if resp.starts_with('=') {
            let s = &resp.trim()[2..];
            // TODO pass and resign
            Ok(MoveAction::Move {
                stone,
                coord: Coord::from_a1(s, self.size).unwrap(),
            })
        } else {
            Err(PlayerError::EngineError(resp))
        }
    }
}
