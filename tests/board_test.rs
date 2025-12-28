use rustgo::backend::{Board, BoardState, Coord, Stone};

const VOID: Stone = Stone::VOID;
const BLACK: Stone = Stone::BLACK;
const WHITE: Stone = Stone::WHITE;

#[test]
fn test_init() {
    let engine = Board::new(3);
    assert_eq!(engine.board(), [VOID; 3 * 3]);
}

#[test]
fn test_1() {
    let mut engine = Board::new(3);
    let _ = engine.place_stone(Coord::new(1, 1), BLACK);
    assert_eq!(
        engine.board_string(),
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
    let board: BoardState = Box::new([
        VOID, BLACK, VOID,
        BLACK, VOID, BLACK,
        VOID, BLACK, VOID,
    ]);
    let mut engine = Board::new_with_board(3, board);
    let result = engine.place_stone(Coord::new(1, 1), BLACK);
    assert!(result.is_ok());
}

#[test]
fn test_3() {
    #[rustfmt::skip]
    let board: BoardState = Box::new([
        VOID, BLACK, VOID,
        BLACK, VOID, BLACK,
        VOID, BLACK, VOID,
    ]);
    let mut engine = Board::new_with_board(3, board);
    let result = engine.place_stone(Coord::new(1, 1), WHITE);
    assert!(result.is_err());
}
