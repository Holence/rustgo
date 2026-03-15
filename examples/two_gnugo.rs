use rustgo::{
    Stone,
    board::Board,
    player::{MoveAction, PlayerTrait, local_gnugo_player::LocalGnugoPlayer},
};

const BOARD_SIZE: usize = 19;

fn main() {
    let mut board = Board::new(BOARD_SIZE);
    let mut player1 = LocalGnugoPlayer::new(BOARD_SIZE).unwrap();
    let mut player2 = LocalGnugoPlayer::new(BOARD_SIZE).unwrap();

    let mut stone = Stone::BLACK;
    loop {
        if stone == Stone::BLACK {
            let move_action = player1.genmove(stone).unwrap();
            match move_action {
                MoveAction::Move { stone, coord } => {
                    board.place_stone(coord, stone).unwrap();
                }
                MoveAction::Pass => todo!(),
                MoveAction::Resign => todo!(),
            }
            player2.play(move_action).unwrap();
        } else {
            let move_action = player2.genmove(stone).unwrap();
            match move_action {
                MoveAction::Move { stone, coord } => {
                    board.place_stone(coord, stone).unwrap();
                }
                MoveAction::Pass => todo!(),
                MoveAction::Resign => todo!(),
            }
            player1.play(move_action).unwrap();
        }
        stone = stone.next_stone(2);
        println!("{}", board.board_string());
    }
}
