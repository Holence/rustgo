use rand::{Rng, rngs::ThreadRng};
use rustgo::{Coord, Stone, board::Board};

fn random_number(rng: &mut ThreadRng, size: usize) -> usize {
    (rng.random::<u32>() % (size as u32)) as usize
}

const N_PLAYER: usize = 6;
const BOARD_SIZE: usize = 42;

fn main() {
    let mut rng = rand::rng(); // a local handle to the generator

    let mut engine = Board::new(BOARD_SIZE);
    let mut stone = Stone::BLACK;
    let mut moves: usize = 0;
    'outer: for _ in 0..1000000 {
        for _ in 0..20 {
            // 对于一种颜色，随机落子多次，如果都不成功，则结束对局
            let res = engine.place_stone(
                Coord::new(
                    random_number(&mut rng, BOARD_SIZE),
                    random_number(&mut rng, BOARD_SIZE),
                ),
                stone,
            );
            if res.is_ok() {
                stone = stone.next_stone(N_PLAYER);
                println!("{:?}", res);
                println!("{}", engine.board_string());
                moves += 1;
                continue 'outer;
            }
        }
        break 'outer;
    }
    println!("{}", engine.board_string());
    println!("moves: {}", moves);
}
