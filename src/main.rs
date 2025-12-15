use rustgo::{engine::Engine, ui::run_front_end};

fn main() {
    let engine = Engine::new(19);
    run_front_end(engine);
}
