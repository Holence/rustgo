use rustgo::backend::{Board, Coord, Engine, Stone};

const VOID: Stone = Stone::VOID;
const BLACK: Stone = Stone::BLACK;
const WHITE: Stone = Stone::WHITE;

#[test]
fn test_init() {
    let engine = Engine::new(3);
    assert_eq!(engine.board(), [VOID; 3 * 3]);
}

#[test]
fn test_1() {
    let mut engine = Engine::new(3);
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
    let board :Board = Box::new([
        VOID, BLACK, VOID,
        BLACK, VOID, BLACK,
        VOID, BLACK, VOID,
    ]);
    let mut engine = Engine::new_with_board(3, board);
    let result = engine.place_stone(Coord::new(1, 1), BLACK);
    assert!(result.is_ok());
}

#[test]
fn test_3() {
    #[rustfmt::skip]
    let board :Board = Box::new([
        VOID, BLACK, VOID,
        BLACK, VOID, BLACK,
        VOID, BLACK, VOID,
    ]);
    let mut engine = Engine::new_with_board(3, board);
    let result = engine.place_stone(Coord::new(1, 1), WHITE);
    assert!(result.is_err());
}
