#![allow(clippy::modulo_one)]
#![allow(clippy::identity_op)]

use cursive::{
    Printer, Vec2,
    direction::Direction,
    event::{Event, EventResult, MouseButton, MouseEvent},
    view::{CannotFocus, View},
    views::TextView,
};

use cursive::traits::Nameable;
use cursive::view::Resizable;
use cursive::views::{Dialog, LinearLayout, Panel};
use rustgo::{Coord, Stone, board::Board};

pub struct BoardView {
    board: Board,
    cur_stone: Stone,
    n_stone: u8,
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
    pub fn new(size: usize, n_stone: u8) -> Self {
        Self {
            board: Board::new(size),
            cur_stone: Stone::BLACK,
            n_stone,
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

        if x < self.board.size() && y < self.board.size() {
            Some(Vec2::new(x, y))
        } else {
            None
        }
    }

    fn place_stone(&mut self, pos: Vec2) -> EventResult {
        let coord = Coord::new(pos.x, pos.y);
        let result = self.board.place_stone(coord, self.cur_stone);
        match result {
            Ok(eaten) => {
                // TODO
                eaten;
                self.cur_stone = self.cur_stone.next_stone(self.n_stone);
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
        let size = self.board.size();
        let board = self.board.board_array();

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
            self.board.size() * CELL_PER_X + BOARD_OFFSET_X,
            self.board.size() * CELL_PER_Y + BOARD_OFFSET_Y,
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

fn main() {
    let mut siv = cursive::default();

    let board = BoardView::new(19, 3);
    let log_view = TextView::new("Log:\n").with_name("log").min_width(30);

    siv.add_layer(
        Dialog::new().title("围棋").content(
            LinearLayout::horizontal()
                .child(Panel::new(board))
                .child(Panel::new(log_view)),
        ),
    );
    siv.add_global_callback('q', |s| s.quit());

    // siv.set_theme(Theme::terminal_default());

    siv.run();
}
