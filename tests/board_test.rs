use rustgo::backend::{Board, Coord, Engine, Stone::Black, Stone::Void, Stone::White};

#[test]
fn test_init() {
    let engine = Engine::new(3);
    assert_eq!(engine.board(), [Void; 3 * 3]);
}

#[test]
fn test_1() {
    let mut engine = Engine::new(3);
    let _ = engine.place_stone(Coord::new(1, 1), Black);
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
        Void, Black, Void,
        Black, Void, Black,
        Void, Black, Void,
    ]);
    let mut engine = Engine::new_with_board(3, board);
    let result = engine.place_stone(Coord::new(1, 1), Black);
    assert!(result.is_ok());
}

#[test]
fn test_3() {
    #[rustfmt::skip]
    let board :Board = Box::new([
        Void, Black, Void,
        Black, Void, Black,
        Void, Black, Void,
    ]);
    let mut engine = Engine::new_with_board(3, board);
    let result = engine.place_stone(Coord::new(1, 1), White);
    assert!(result.is_err());
}
