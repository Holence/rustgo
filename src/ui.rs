use cursive::{
    Cursive, CursiveExt,
    event::Key,
    view::Nameable,
    views::{Dialog, TextView},
};

use crate::engine::{Engine, Stone};

struct GameState {
    engine: Engine,
    cursor_y: usize,
    cursor_x: usize,
    next_stone: Stone,
}

impl GameState {
    fn new(engine: Engine) -> Self {
        Self {
            engine,
            cursor_y: 0,
            cursor_x: 0,
            next_stone: Stone::Black,
        }
    }
}

impl Stone {
    fn char(self) -> char {
        match self {
            Stone::Black => '●',
            Stone::White => '○',
        }
    }

    fn next(self) -> Self {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
        }
    }
}

// 19x19棋盘的星位
const STAR: [(usize, usize); 9] = [
    (3, 3),
    (3, 15),
    (15, 3),
    (15, 15),
    (3, 9),
    (9, 3),
    (15, 9),
    (9, 15),
    (9, 9),
];

fn refresh_board(siv: &mut Cursive) {
    let gs: &mut GameState = siv.user_data().unwrap();
    let engine = &gs.engine;
    let width = engine.width();
    let board = engine.board();

    let mut out = String::new();

    for y in 0..width {
        out.push(' ');
        for x in 0..width {
            if x == gs.cursor_x && y == gs.cursor_y {
                out.push('x');
            } else {
                match board[y * width + x] {
                    Some(stone) => out.push(stone.char()),
                    None => {
                        if width == 19 && STAR.contains(&(y, x)) {
                            // 星位
                            out.push('+');
                        } else {
                            out.push('·')
                        }
                    }
                }
            }
            out.push(' ');
        }
        out.push('\n');
    }

    out.push_str("\n");
    out.push_str(match gs.next_stone {
        Stone::Black => "Turn: Black (●)",
        Stone::White => "Turn: White (○)",
    });

    siv.call_on_name("board", |v: &mut TextView| {
        v.set_content(out);
    });
}

fn move_cursor(siv: &mut Cursive, dy: isize, dx: isize) {
    let gs: &mut GameState = siv.user_data().unwrap();
    let width = gs.engine.width();
    gs.cursor_y = gs.cursor_y.saturating_add_signed(dy).clamp(0, width - 1);
    gs.cursor_x = gs.cursor_x.saturating_add_signed(dx).clamp(0, width - 1);
    refresh_board(siv);
}

fn place_stone(siv: &mut Cursive) {
    let result = siv
        .with_user_data(|gs: &mut GameState| {
            gs.engine
                .place_stone(gs.cursor_y, gs.cursor_x, gs.next_stone)
        })
        .unwrap();

    match result {
        Err(msg) => {
            siv.add_layer(Dialog::info(msg));
        }
        Ok(()) => {
            siv.with_user_data(|gs: &mut GameState| {
                gs.next_stone = gs.next_stone.next();
            });
            refresh_board(siv);
        }
    }
}

pub fn run_front_end(engine: Engine) {
    let mut siv = Cursive::new();

    siv.set_user_data(GameState::new(engine));
    siv.add_layer(TextView::new("").with_name("board"));

    // Quit
    siv.add_global_callback('q', |s| s.quit());

    // Cursor movement
    siv.add_global_callback(Key::Up, |s| move_cursor(s, -1, 0));
    siv.add_global_callback(Key::Down, |s| move_cursor(s, 1, 0));
    siv.add_global_callback(Key::Left, |s| move_cursor(s, 0, -1));
    siv.add_global_callback(Key::Right, |s| move_cursor(s, 0, 1));

    // Space to Place stone
    siv.add_global_callback(' ', |s| place_stone(s));

    refresh_board(&mut siv);

    siv.run();
}
