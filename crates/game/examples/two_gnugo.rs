use game::{
    Action,
    player::{PlayerTrait, local_gnugo_player::LocalGnugoPlayer},
};
use rustgo::{Stone, board::Board};

const BOARD_SIZE: usize = 9;

fn main() {
    let mut board = Board::new(BOARD_SIZE);
    let mut player1 = LocalGnugoPlayer::new(BOARD_SIZE).unwrap();
    let mut player2 = LocalGnugoPlayer::new(BOARD_SIZE).unwrap();

    let mut stone = Stone::BLACK;
    loop {
        if stone == Stone::BLACK {
            let action = player1.genmove(stone).unwrap();
            match action {
                Action::Move { stone, coord } => {
                    board.place_stone(coord, stone).unwrap();
                    player1.play(stone, coord).unwrap();
                    player2.play(stone, coord).unwrap();
                }
                Action::Pass => todo!(),
                Action::Resign => todo!(),
            }
        } else {
            let action = player2.genmove(stone).unwrap();
            match action {
                Action::Move { stone, coord } => {
                    board.place_stone(coord, stone).unwrap();
                    player1.play(stone, coord).unwrap();
                    player2.play(stone, coord).unwrap();
                }
                Action::Pass => todo!(),
                Action::Resign => todo!(),
            }
        }
        stone = stone.next_stone(2);
        println!("{}", board.board_string());
    }
}
