use rustgo::{Coord, Stone, board::Board};

fn main() {
    let mut board = Board::new(19);
    board.place_stone(Coord::new(3, 3), Stone::BLACK).unwrap();
    println!("{}", board.board_string());
}
