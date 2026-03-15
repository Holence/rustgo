use rustgo::{
    Coord, Stone,
    board::{Board, BoardArray},
};

const VOID: Stone = Stone::VOID;
const BLACK: Stone = Stone::BLACK;
const WHITE: Stone = Stone::WHITE;

#[test]
fn test_init() {
    let board = Board::new(3);
    assert_eq!(board.board_array(), [VOID; 3 * 3]);
}

#[test]
fn test_1() {
    let mut board = Board::new(3);
    let _ = board.place_stone(Coord::new(1, 1), BLACK);
    assert_eq!(
        board.board_string(),
        "\
___
_●_
___
"
    );
}

#[test]
fn test_2() {
    #[rustfmt::skip]
    let board: BoardArray = Box::new([
        VOID, BLACK, VOID,
        BLACK, VOID, BLACK,
        VOID, BLACK, VOID,
    ]);
    let mut board = Board::new_with_board(3, board);
    let result = board.place_stone(Coord::new(1, 1), BLACK);
    assert!(result.is_ok());
}

#[test]
fn test_3() {
    #[rustfmt::skip]
    let board: BoardArray = Box::new([
        VOID, BLACK, VOID,
        BLACK, VOID, BLACK,
        VOID, BLACK, VOID,
    ]);
    let mut board = Board::new_with_board(3, board);
    let result = board.place_stone(Coord::new(1, 1), WHITE);
    assert!(result.is_err());
}
