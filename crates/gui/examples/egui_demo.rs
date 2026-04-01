#[derive(Clone, Copy, PartialEq)]
enum Stone {
    Black,
    White,
}

struct GoBoard {
    size: usize, // 19
    grid: Vec<Option<Stone>>,
    next: Stone,
}

impl GoBoard {
    fn new(size: usize) -> Self {
        Self {
            size,
            grid: vec![None; size * size],
            next: Stone::Black,
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.size + x
    }

    fn place(&mut self, x: usize, y: usize) {
        let i = self.idx(x, y);
        if self.grid[i].is_none() {
            self.grid[i] = Some(self.next);
            self.next = match self.next {
                Stone::Black => Stone::White,
                Stone::White => Stone::Black,
            };
        }
    }
}

use eframe::egui::{self};

impl GoBoard {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let board_size_px = 300.0; // total board size
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(board_size_px, board_size_px),
            egui::Sense::click(),
        );

        let painter = ui.painter_at(rect);

        let n = self.size as f32;
        let cell = rect.width() / (n - 1.0);

        // --- draw grid ---
        for i in 0..self.size {
            let t = i as f32;

            // vertical line
            painter.line_segment(
                [
                    rect.left_top() + egui::vec2(t * cell, 0.0),
                    rect.left_top() + egui::vec2(t * cell, rect.height()),
                ],
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            );

            // horizontal line
            painter.line_segment(
                [
                    rect.left_top() + egui::vec2(0.0, t * cell),
                    rect.left_top() + egui::vec2(rect.width(), t * cell),
                ],
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            );
        }

        // --- handle click ---
        if let Some(pos) = response.interact_pointer_pos()
            && response.clicked()
        {
            // convert pixel → board coord
            let local = pos - rect.min;

            let x = (local.x / cell).round() as isize;
            let y = (local.y / cell).round() as isize;

            if x >= 0 && y >= 0 && x < self.size as isize && y < self.size as isize {
                self.place(x as usize, y as usize);
            }
        }

        // --- draw stones ---
        let radius = cell * 0.4;

        for y in 0..self.size {
            for x in 0..self.size {
                if let Some(stone) = self.grid[self.idx(x, y)] {
                    let center = rect.left_top() + egui::vec2(x as f32 * cell, y as f32 * cell);

                    let color = match stone {
                        Stone::Black => egui::Color32::BLACK,
                        Stone::White => egui::Color32::WHITE,
                    };

                    painter.circle_filled(center, radius, color);

                    // optional: outline for white stones
                    if stone == Stone::White {
                        painter.circle_stroke(
                            center,
                            radius,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                        );
                    }
                }
            }
        }
    }
}

struct MyApp {
    board: GoBoard,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            board: GoBoard::new(19),
            show_confirmation_dialog: false,
            allowed_to_close: false,
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.board.ui(ui);
        });

        if ui.input(|i| i.viewport().close_requested()) {
            if self.allowed_to_close {
                // do nothing - we will close
            } else {
                ui.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_confirmation_dialog = true;
            }
        }

        if self.show_confirmation_dialog {
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("No").clicked() {
                            self.show_confirmation_dialog = false;
                            self.allowed_to_close = false;
                        }

                        if ui.button("Yes").clicked() {
                            self.show_confirmation_dialog = false;
                            self.allowed_to_close = true;
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Go Board Demo",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Light);

            Ok(Box::new(MyApp::default()))
        }),
    )
}
