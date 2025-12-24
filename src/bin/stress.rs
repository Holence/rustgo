use cursive::reexports::log::debug;
use rand::{Rng, rngs::ThreadRng};
use rustgo::backend::{Coord, Engine, Stone};

fn random_number(rng: &mut ThreadRng, size: usize) -> usize {
    (rng.random::<u32>() % (size as u32)) as usize
}
fn main() {
    let mut rng = rand::rng(); // a local handle to the generator
    let size = 128;

    let mut engine = Engine::new(size);
    let mut stone = Stone::Black;
    let mut steps: usize = 0;
    'outer: for _ in 0..1000000 {
        for _ in 0..20 {
            // 对于一种颜色，随机落子多次，如果都不成功，则结束对局
            let res = engine.place_stone(
                Coord::new(random_number(&mut rng, size), random_number(&mut rng, size)),
                stone,
            );
            if res.is_ok() {
                if stone == Stone::Black {
                    stone = Stone::White;
                } else {
                    stone = Stone::Black;
                }

                let _ = debug!("{:?}", res);
                debug!("{}", engine.board_string());
                steps += 1;
                continue 'outer;
            }
        }
        break 'outer;
    }
    println!("{}", engine.board_string());
    println!("steps: {}", steps);
}
