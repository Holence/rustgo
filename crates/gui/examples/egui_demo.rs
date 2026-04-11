use std::sync::{Arc, Mutex};

use eframe::egui::{self, Color32, Vec2};
use game::{
    Action, PlayerMessage, ServerMessage,
    game::GameBuilder,
    player::{PlayerId, channel_player::ChannelPlayer, dummy_player::DummyPlayer},
    team::TeamId,
};
use rustgo::{Coord, Stone, board::Board};
use tokio::sync::mpsc::{self, Receiver, Sender};

static COLOR32_LUT: &[Color32] = &[
    Color32::TRANSPARENT,
    Color32::BLACK,
    Color32::WHITE,
    Color32::BROWN,
    Color32::RED,
    Color32::LIGHT_RED,
    Color32::CYAN,
    Color32::MAGENTA,
    Color32::YELLOW,
    Color32::ORANGE,
    Color32::LIGHT_YELLOW,
    Color32::KHAKI,
    Color32::DARK_GREEN,
    Color32::GREEN,
    Color32::LIGHT_GREEN,
    Color32::DARK_BLUE,
    Color32::BLUE,
    Color32::LIGHT_BLUE,
    Color32::PURPLE,
    Color32::GOLD,
];

struct UiBoard {
    player_id: PlayerId,
    size: usize,
    board: Board,
    pending_move: Option<Stone>,
    ui_tx: Sender<PlayerMessage>, // 点击事件，发出信息
}

impl UiBoard {
    fn new(player_id: PlayerId, size: usize, ui_tx: Sender<PlayerMessage>) -> Self {
        Self {
            player_id,
            size,
            board: Board::new(size),
            pending_move: None,
            ui_tx,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let board_size_px = ui.available_size().min_elem();

        let (response, painter) = ui.allocate_painter(
            Vec2 {
                x: board_size_px,
                y: board_size_px,
            },
            egui::Sense::click(),
        );
        let rect = response.rect;

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
        if let Some(stone) = self.pending_move
            && let Some(pos) = response.interact_pointer_pos()
            && response.clicked()
        {
            // convert pixel → board coord
            let local = pos - rect.min;

            let x = (local.x / cell).round() as isize;
            let y = (local.y / cell).round() as isize;

            if x >= 0 && y >= 0 && x < self.size as isize && y < self.size as isize {
                let coord = Coord::new(x as usize, y as usize);

                self.ui_tx
                    .try_send(PlayerMessage::PlayerAction {
                        player_id: self.player_id,
                        action: Action::Move { stone, coord },
                    })
                    .unwrap();

                self.pending_move = None;
            }
        }

        // --- draw stones ---
        let radius = cell * 0.4;
        let board = self.board.board_array();

        for y in 0..self.size {
            for x in 0..self.size {
                let idx = y * self.size + x;
                let stone = board[idx];
                if stone != Stone::VOID {
                    let center = rect.left_top() + egui::vec2(x as f32 * cell, y as f32 * cell);

                    let color = COLOR32_LUT[stone.as_usize()];

                    painter.circle_filled(center, radius, color);

                    // outline
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

struct MyApp {
    board_ui: Arc<Mutex<UiBoard>>,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
}

impl MyApp {
    pub fn new(
        cc: &eframe::CreationContext,
        player_id: PlayerId,
        size: usize,
        ui_tx: Sender<PlayerMessage>,
        mut ui_rx: Receiver<ServerMessage>,
    ) -> Self {
        let board_ui = Arc::new(Mutex::new(UiBoard::new(player_id, size, ui_tx)));

        {
            let ctx = cc.egui_ctx.clone();
            let board_ui = board_ui.clone();
            tokio::spawn(async move {
                while let Some(msg) = ui_rx.recv().await {
                    match msg {
                        ServerMessage::GameStart(team_infos) => todo!(),
                        ServerMessage::GameUpdate {
                            cur_team,
                            cur_player,
                            player_info,
                        } => {
                            // TODO
                        }
                        ServerMessage::PlayerMove {
                            player_id,
                            stone,
                            coord,
                        } => {
                            board_ui
                                .lock()
                                .unwrap()
                                .board
                                .place_stone(coord, stone)
                                .unwrap();
                        }
                        ServerMessage::PlayerChat { player_id, chat } => {
                            println!("egui hear {} from player[{:?}]", chat, player_id)
                        }
                        ServerMessage::GenMove(stone) => {
                            board_ui.lock().unwrap().pending_move = Some(stone)
                        }
                        ServerMessage::Error(msg) => {
                            println!("ServerMessage::Error: {}", msg);
                        }
                        ServerMessage::GameOver => todo!(),
                    }
                    ctx.request_repaint();
                }
            });
        }

        Self {
            board_ui,
            show_confirmation_dialog: false,
            allowed_to_close: false,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        self.board_ui.lock().unwrap().ui(ui)
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.ui(ui);
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

const BOARD_SIZE: usize = 19;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let (ui_tx, uplink_from_ui) = mpsc::channel::<PlayerMessage>(32);
    let (downlink_to_ui, ui_rx) = mpsc::channel::<ServerMessage>(32);

    tokio::spawn(async move {
        let mut game = GameBuilder::new(BOARD_SIZE);
        game.add_team(TeamId::new(0), Stone::BLACK);
        game.add_player(
            TeamId::new(0),
            ChannelPlayer::new(PlayerId::new(0), downlink_to_ui, uplink_from_ui),
        );

        for team_n in 1..6 {
            game.add_team(TeamId::new(team_n), Stone::new(1 + team_n as u8));
            for player_n in 0..team_n {
                game.add_player(
                    TeamId::new(team_n),
                    DummyPlayer::new(PlayerId::new(team_n * 10 + player_n), BOARD_SIZE),
                );
            }
        }

        let mut game = game.build();
        game.run().await;
    });

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Go Board Demo",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Light);
            Ok(Box::new(MyApp::new(
                cc,
                PlayerId::new(0),
                BOARD_SIZE,
                ui_tx,
                ui_rx,
            )))
        }),
    )
}
