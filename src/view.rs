#![allow(clippy::modulo_one)]
#![allow(clippy::identity_op)]

use cursive::{
    Printer, Vec2,
    direction::Direction,
    event::{Event, EventResult, MouseButton, MouseEvent},
    view::{CannotFocus, View},
    views::TextView,
};

use crate::{
    backend::{Coord, Stone},
    model::Game,
};

pub struct BoardView {
    game: Game,
}

const X_TIMES: usize = 1; // you can modify this
const CELL_PER_X: usize = 2 * X_TIMES; // don't modify this
const CELL_PER_Y: usize = 1 * X_TIMES; // don't modify this
const BOARD_OFFSET_X: usize = CELL_PER_X - 1; // don't modify this
const BOARD_OFFSET_Y: usize = CELL_PER_Y - 1; // don't modify this

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

impl BoardView {
    pub fn new(size: usize, n_player: usize) -> Self {
        Self {
            game: Game::new(size, n_player),
        }
    }

    fn position_to_cell(&self, mouse: Vec2, offset: Vec2) -> Option<Vec2> {
        let pos = mouse.checked_sub(offset)?;
        let pos = pos.checked_sub(Vec2::new(BOARD_OFFSET_X, BOARD_OFFSET_Y))?;

        if pos.x % CELL_PER_X != 0 || pos.y % CELL_PER_Y != 0 {
            return None;
        }
        let x = pos.x / CELL_PER_X;
        let y = pos.y / CELL_PER_Y;

        if x < self.game.size() && y < self.game.size() {
            Some(Vec2::new(x, y))
        } else {
            None
        }
    }

    fn place_stone(&mut self, pos: Vec2) -> EventResult {
        let coord = Coord::new(pos.x, pos.y);
        let result = self.game.place_stone(coord);
        match result {
            Ok(action) => {
                // TODO
                action.eaten_stones;
                EventResult::with_cb_once(move |s| append_log(s, coord.to_string()))
            }
            Err(msg) => EventResult::with_cb(move |s| {
                s.add_layer(cursive::views::Dialog::info(msg));
            }),
        }
    }
}

impl View for BoardView {
    fn draw(&self, printer: &Printer) {
        let size = self.game.size();
        let board = self.game.board();

        let mut line = String::with_capacity(size);
        for y in 0..size {
            line.clear();
            line.push(' ');
            for x in 0..size {
                let idx = y * size + x;
                let c = match board[idx] {
                    Stone::VOID => {
                        if size == 19 && STAR.contains(&(x, y)) {
                            '+'
                        } else {
                            '·'
                        }
                    }
                    stone => stone.as_char(),
                };
                line.push(c);
                line.push(' ');
            }
            printer.print((0, y * CELL_PER_Y + BOARD_OFFSET_Y), &line);
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        Vec2::new(
            self.game.size() * CELL_PER_X + BOARD_OFFSET_X,
            self.game.size() * CELL_PER_Y + BOARD_OFFSET_Y,
        )
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Mouse {
                offset,
                position,
                event: MouseEvent::Release(MouseButton::Left),
            } => {
                if let Some(pos) = self.position_to_cell(position, offset) {
                    return self.place_stone(pos);
                }
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}

fn append_log(s: &mut cursive::Cursive, mut msg: String) {
    msg.push('\n');
    s.call_on_name("log", |view: &mut TextView| {
        view.append(msg);
    });
}
