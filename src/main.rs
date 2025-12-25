use cursive::traits::Nameable;
use cursive::view::Resizable;
use cursive::views::{Dialog, LinearLayout, Panel, TextView};
use rustgo::view::BoardView;

fn main() {
    let mut siv = cursive::default();

    let board = BoardView::new(19, 2);
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
