use std::time::Duration;

use eframe::egui::{self, Color32, Vec2};
use game::{
    Action, PlayerMessage, ServerMessage,
    game::{Game, GameBuilder},
    player::{
        PlayerId, channel_player::ChannelPlayer, dummy_player::DummyPlayer,
        local_gnugo_player::LocalGnugoPlayer,
    },
    team::TeamId,
};
use rustgo::{Coord, Stone, board::Board};
use tokio::sync::mpsc::{self, Receiver, Sender};

struct UiBoard {
    board: Board,
    pending_move: Option<Stone>,
    ui_tx: Sender<PlayerMessage>,   // 点击事件，发出信息
    ui_rx: Receiver<ServerMessage>, // 接收信息，更新棋盘
}

static COLOR32_LUT: &[Color32] = &[
    Color32::TRANSPARENT,
    Color32::BLACK,
    Color32::WHITE,
    Color32::BROWN,
    Color32::DARK_RED,
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

impl UiBoard {
    fn new(size: usize, ui_tx: Sender<PlayerMessage>, ui_rx: Receiver<ServerMessage>) -> Self {
        Self {
            board: Board::new(size),
            pending_move: None,
            ui_tx,
            ui_rx,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let board_size_px = ui.available_size().min_elem();
        let size = self.board.size();

        if let Ok(msg) = self.ui_rx.try_recv() {
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
                    self.board.place_stone(coord, stone).unwrap();
                }
                ServerMessage::PlayerChat { player_id, chat } => {
                    println!("egui hear {} from player[{:?}]", chat, player_id)
                }
                ServerMessage::GenMove(stone) => self.pending_move = Some(stone),
                ServerMessage::Error(_) => todo!(),
                ServerMessage::GameOver => todo!(),
            }
        }

        let (response, painter) = ui.allocate_painter(
            Vec2 {
                x: board_size_px,
                y: board_size_px,
            },
            egui::Sense::click(),
        );
        let rect = response.rect;

        let n = size as f32;
        let cell = rect.width() / (n - 1.0);

        // --- draw grid ---
        for i in 0..size {
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

            if x >= 0 && y >= 0 && x < size as isize && y < size as isize {
                let coord = Coord::new(x as usize, y as usize);

                self.ui_tx
                    .try_send(PlayerMessage::PlayerAction {
                        player_id: PlayerId::new(0), // TODO replace me
                        action: Action::Move { stone, coord },
                    })
                    .unwrap();

                self.pending_move = None;
            }
        }

        // --- draw stones ---
        let radius = cell * 0.4;
        let board = self.board.board_array();

        for y in 0..size {
            for x in 0..size {
                let idx = y * size + x;
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
    board: UiBoard,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
}

impl MyApp {
    pub fn new(size: usize, ui_tx: Sender<PlayerMessage>, ui_rx: Receiver<ServerMessage>) -> Self {
        Self {
            board: UiBoard::new(size, ui_tx, ui_rx),
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
            PlayerId::new(0),
            "egui".to_string(),
            ChannelPlayer::new(downlink_to_ui, uplink_from_ui),
        );
        game.add_player(
            TeamId::new(0),
            PlayerId::new(1),
            "GnuGo0".to_string(),
            LocalGnugoPlayer::new(BOARD_SIZE).unwrap(),
        );

        game.add_team(TeamId::new(1), Stone::WHITE);
        game.add_player(
            TeamId::new(1),
            PlayerId::new(10),
            "GnuGo1".to_string(),
            LocalGnugoPlayer::new(BOARD_SIZE).unwrap(),
        );
        game.add_player(
            TeamId::new(1),
            PlayerId::new(11),
            "Dummy".to_string(),
            DummyPlayer::new(BOARD_SIZE),
        );

        let mut game = game.build();
        game.run().await;
    });

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Go Board Demo",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Light);
            Ok(Box::new(MyApp::new(BOARD_SIZE, ui_tx, ui_rx)))
        }),
    )

    // TODO ui_rx 放到另一个协程中，处理消息，更新MyApp状态，然后 ui.request_repaint()？ 现在消息来了ui是不会被唤醒的，界面也不会被更新
}
