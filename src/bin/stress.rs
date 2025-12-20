use rand::{Rng, rngs::ThreadRng};
use rustgo::backend::{Coord, Engine, Stone};

fn random_number(rng: &mut ThreadRng, size: usize) -> usize {
    (rng.random::<u32>() % (size as u32)) as usize
}
fn main() {
    let mut rng = rand::rng(); // a local handle to the generator
    let size = 128;

    let mut e = Engine::new(size);
    let mut stone = Stone::Black;
    for _ in 0..1000000 {
        let res = e.place_stone(
            Coord::new(random_number(&mut rng, size), random_number(&mut rng, size)),
            stone,
        );
        // if let Ok(res) = res {
        //     if res.eaten.len() > 0 {
        //         println!("{:?}", res.eaten);
        //     }
        // }
        if stone == Stone::Black {
            stone = Stone::White;
        } else {
            stone = Stone::Black;
        }
    }
    // println!("{}", e.board_string());
}
