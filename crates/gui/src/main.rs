use gui::network_task::{NetworkTaskCmd, NetworkTaskEvent, network_task};
use server::common::{ClientId, ReqId, RoomId, UplinkMessage};
use std::fmt::Debug;
use tokio::sync::mpsc;

use eframe::egui;

#[derive(Default, Debug, Clone)]
struct LobbyState {
    pub(crate) chat_input: String,
    pub(crate) chat_log: Vec<String>,
}

#[derive(Debug, Clone)]
enum ViewState {
    Home,
    GoingToLobby,
    Lobby(LobbyState),
    Room(RoomId),
}

#[derive(Debug)]
struct Pending {
    req_id: ReqId,
    description: String,
}

#[derive(Debug)]
enum UiAction {
    Connect,
    Disconnect,
    SendLobbyChat(String),
    // future:
    // EnterRoom(RoomId),
    // CreateRoom,
}

pub struct App {
    state: ViewState,
    pending: Option<Pending>,
    next_req_id: ReqId,
    client_id: Option<ClientId>,

    tx_cmd: mpsc::UnboundedSender<NetworkTaskCmd>,
    rx_msg: mpsc::UnboundedReceiver<NetworkTaskEvent>,
}

impl App {
    fn new(
        tx_cmd: mpsc::UnboundedSender<NetworkTaskCmd>,
        rx_msg: mpsc::UnboundedReceiver<NetworkTaskEvent>,
    ) -> Self {
        Self {
            state: ViewState::Home,
            pending: None,
            next_req_id: 1,
            client_id: None,
            tx_cmd,
            rx_msg,
        }
    }

    fn next_req(&mut self) -> ReqId {
        let id = self.next_req_id;
        self.next_req_id += 1;
        id
    }

    fn change_state(&mut self, state: ViewState) {
        self.state = state;
        self.pending = None;
    }

    fn send(&mut self, msg: UplinkMessage) {
        self.tx_cmd.send(NetworkTaskCmd::Send(msg)).unwrap();
    }

    fn connect(&mut self) {
        self.tx_cmd.send(NetworkTaskCmd::Connect).unwrap();
    }

    fn disconnect(&mut self) {
        self.tx_cmd.send(NetworkTaskCmd::Disconnect).unwrap();
    }

    fn pending_matches(&self, req_id: ReqId) -> bool {
        if let Some(pending) = &self.pending {
            return pending.req_id == req_id;
        }
        return false;
    }

    fn handle_home(&mut self) {
        while let Ok(event) = self.rx_msg.try_recv() {
            match event {
                NetworkTaskEvent::Connected => {
                    self.change_state(ViewState::GoingToLobby);
                    break;
                }
                _ => unreachable!("{:?}", event),
            }
        }
    }

    fn ui_home(&mut self, ui: &mut egui::Ui) -> Option<UiAction> {
        let mut action: Option<UiAction> = None;
        ui.heading("Home");
        ui.separator();

        if ui.button("Connect").clicked() {
            action = Some(UiAction::Connect);
        }
        return action;
    }

    fn handle_going_to_lobby(&mut self) {
        while let Ok(event) = self.rx_msg.try_recv() {
            match event {
                NetworkTaskEvent::Disconnected => {
                    self.change_state(ViewState::Home);
                    break;
                }
                NetworkTaskEvent::Recv(downlink_message) => {
                    let req_id = downlink_message.req_id;
                    match downlink_message.msg {
                        server::common::DownlinkMessageValue::Greeting(client_id) => {
                            self.client_id = Some(client_id);
                            let req_id = self.next_req();
                            self.pending = Some(Pending {
                                req_id,
                                description: "Enter Lobby".to_string(),
                            });
                            self.send(UplinkMessage::LobbyEnter { client_id, req_id });
                        }
                        server::common::DownlinkMessageValue::Lobby(downlink_lobby_message) => {
                            match downlink_lobby_message {
                                server::common::DownlinkLobbyMessage::EnterAck { success } => {
                                    if self.pending_matches(req_id) {
                                        self.pending = None;
                                        if success {
                                            self.change_state(ViewState::Lobby(
                                                LobbyState::default(),
                                            ));
                                            break;
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    fn ui_going_to_lobby(&mut self, ui: &mut egui::Ui) -> Option<UiAction> {
        let action: Option<UiAction> = None;
        ui.heading("Going To Lobby...");
        return action;
    }

    fn handle_lobby(&mut self) {
        let ViewState::Lobby(lobby_state) = &mut self.state else {
            unreachable!()
        };

        while let Ok(event) = self.rx_msg.try_recv() {
            match event {
                NetworkTaskEvent::Disconnected => {
                    self.change_state(ViewState::Home);
                    break;
                }
                NetworkTaskEvent::Recv(downlink_message) => {
                    let req_id = downlink_message.req_id;
                    match downlink_message.msg {
                        server::common::DownlinkMessageValue::Lobby(downlink_lobby_message) => {
                            match downlink_lobby_message {
                                server::common::DownlinkLobbyMessage::Chat {
                                    client_id,
                                    content,
                                } => {
                                    lobby_state.chat_log.push(format!("{client_id}: {content}"));
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    fn ui_lobby(&mut self, ui: &mut egui::Ui) -> Option<UiAction> {
        let ViewState::Lobby(lobby_state) = &mut self.state else {
            unreachable!()
        };

        let mut action: Option<UiAction> = None;
        ui.heading("Lobby");
        ui.separator();
        ui.label(format!("client_id: {}", self.client_id.unwrap()));
        if ui.button("Disconnect").clicked() {
            action = Some(UiAction::Disconnect);
        }

        ui.label("Chat");

        let response = ui.text_edit_singleline(&mut lobby_state.chat_input);

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let content = lobby_state.chat_input.clone();
            lobby_state.chat_input.clear();

            action = Some(UiAction::SendLobbyChat(content));
        }

        for line in &lobby_state.chat_log {
            ui.label(line);
        }
        return action;
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // -------------------------
        // 1. Drain server messages
        // -------------------------
        match &mut self.state {
            ViewState::Home => self.handle_home(),
            ViewState::GoingToLobby => self.handle_going_to_lobby(),
            ViewState::Lobby(_) => self.handle_lobby(),
            ViewState::Room(_) => todo!(),
        }

        // -------------------------
        // 2. UI → collect action
        // -------------------------
        let mut action: Option<UiAction> = None;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(p) = &self.pending {
                ui.label(format!("Pending: {}", p.description));
            }

            action = match &self.state {
                ViewState::Home => self.ui_home(ui),
                ViewState::GoingToLobby => self.ui_going_to_lobby(ui),
                ViewState::Lobby(_) => self.ui_lobby(ui),
                ViewState::Room(_) => todo!(),
            }
        });

        // -------------------------
        // 3. Execute action
        // -------------------------
        if let Some(action) = action {
            match action {
                UiAction::Connect => {
                    self.connect();
                }

                UiAction::Disconnect => {
                    self.disconnect();
                }

                UiAction::SendLobbyChat(content) => {
                    self.send(UplinkMessage::LobbyChat {
                        client_id: self.client_id.unwrap(),
                        content,
                    });
                }
            }
        }

        ui.request_repaint();
    }
}

#[tokio::main]
async fn main() {
    let (tx_cmd, rx_cmd) = mpsc::unbounded_channel();
    let (tx_msg, rx_msg) = mpsc::unbounded_channel();

    tokio::spawn(network_task("127.0.0.1:8080".to_string(), rx_cmd, tx_msg));

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Client",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(tx_cmd, rx_msg)))),
    )
    .unwrap();
}
