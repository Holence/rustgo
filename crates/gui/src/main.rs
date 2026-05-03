use gui::network_task::{NetworkTaskCmd, NetworkTaskEvent, network_task};
use server::{
    common::{ChatRecord, ClientId, DownlinkMessage, ReqId, RoomId, UplinkMessage},
    lobby::RoomRecord,
};
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::mpsc;

use eframe::egui::{self};

#[derive(Default, Debug, Clone)]
struct LobbyState {
    pub(crate) chat_input: String,
    pub(crate) chats: Vec<ChatRecord>,
    pub(crate) rooms: HashMap<RoomId, RoomRecord>,
    pub(crate) create_room_dialog_open: bool,
    pub(crate) create_room_name_input: String,
}

impl LobbyState {
    fn new(chats: Vec<ChatRecord>, rooms: HashMap<RoomId, RoomRecord>) -> Self {
        Self {
            chat_input: String::new(),
            chats,
            rooms,
            create_room_dialog_open: false,
            create_room_name_input: String::new(),
        }
    }
}

#[derive(Default, Debug, Clone)]
struct RoomState {
    pub(crate) room_id: RoomId,
    pub(crate) chat_input: String,
    pub(crate) chats: Vec<ChatRecord>,
}

#[derive(Debug, Clone)]
enum ViewState {
    Home,
    GoingToLobby,
    Lobby(LobbyState),
    Room(RoomState),
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
    CreateRoom(String),
    EnterRoom(RoomId),
    SendRoomChat(String),
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

    fn next_req(&mut self, description: String) -> ReqId {
        assert!(self.pending.is_none());

        let req_id = self.next_req_id;
        self.pending = Some(Pending {
            req_id,
            description,
        });
        self.next_req_id += 1;
        return req_id;
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
        if let Ok(event) = self.rx_msg.try_recv() {
            match event {
                NetworkTaskEvent::Connected => {
                    self.change_state(ViewState::GoingToLobby);
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
        if let Ok(event) = self.rx_msg.try_recv() {
            match event {
                NetworkTaskEvent::Disconnected => {
                    self.change_state(ViewState::Home);
                }
                NetworkTaskEvent::Recv(msg) => match msg {
                    DownlinkMessage::Greeting { client_id } => {
                        self.client_id = Some(client_id);

                        let req_id = self.next_req("Going To Lobby".to_string());
                        self.send(UplinkMessage::LobbyEnter { client_id, req_id });
                    }
                    DownlinkMessage::LobbyEnterAck {
                        req_id,
                        success,
                        chats,
                        rooms,
                    } => {
                        if self.pending_matches(req_id) {
                            if success {
                                self.change_state(ViewState::Lobby(LobbyState::new(chats, rooms)));
                            } else {
                                todo!()
                            }
                        }
                    }
                    _ => unreachable!("{:?}", msg),
                },
                _ => unreachable!("{:?}", event),
            }
        }
    }

    fn ui_going_to_lobby(&mut self, ui: &mut egui::Ui) -> Option<UiAction> {
        let action: Option<UiAction> = None;
        ui.label(format!("client_id: {:?}", self.client_id));
        return action;
    }

    fn handle_lobby(&mut self) {
        let ViewState::Lobby(lobby_state) = &mut self.state else {
            unreachable!("{:?}", self.state)
        };

        if let Ok(event) = self.rx_msg.try_recv() {
            match event {
                NetworkTaskEvent::Disconnected => {
                    self.change_state(ViewState::Home);
                }
                NetworkTaskEvent::Recv(msg) => match msg {
                    DownlinkMessage::LobbyChatUpdate { chat_record } => {
                        lobby_state.chats.push(chat_record);
                    }
                    DownlinkMessage::LobbyRoomUpdate { room_record } => {
                        lobby_state.rooms.insert(room_record.room_id, room_record);
                    }
                    DownlinkMessage::LobbyCreateRoomAck { req_id, room_id } => {
                        if self.pending_matches(req_id)
                            && let Some(room_id) = room_id
                        {
                            self.change_state(ViewState::Room(RoomState {
                                room_id,
                                chat_input: String::new(),
                                chats: vec![],
                            }));
                        }
                    }
                    DownlinkMessage::RoomEnterAck {
                        req_id,
                        success,
                        room_id,
                        chats,
                    } => {
                        if self.pending_matches(req_id) {
                            if success {
                                self.change_state(ViewState::Room(RoomState {
                                    room_id,
                                    chat_input: String::new(),
                                    chats,
                                }));
                            } else {
                                todo!()
                            }
                        }
                    }
                    _ => unreachable!("{:?}", msg),
                },
                _ => unreachable!("{:?}", event),
            }
        }
    }

    fn ui_lobby(&mut self, ui: &mut egui::Ui) -> Option<UiAction> {
        let ViewState::Lobby(lobby_state) = &mut self.state else {
            unreachable!("{:?}", self.state)
        };

        let mut action: Option<UiAction> = None;
        ui.heading("Lobby");
        ui.separator();
        ui.label(format!("client_id: {}", self.client_id.unwrap()));
        if ui.button("Disconnect").clicked() {
            action = Some(UiAction::Disconnect);
        }
        if ui.button("Create Room").clicked() {
            lobby_state.create_room_dialog_open = true;
        }

        ui.label("Chat");

        let response = ui.text_edit_singleline(&mut lobby_state.chat_input);

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let content = lobby_state.chat_input.clone();
            lobby_state.chat_input.clear();

            action = Some(UiAction::SendLobbyChat(content));
        }

        for chat_record in &lobby_state.chats {
            ui.label(format!(
                "client[{}]: {}",
                chat_record.client_id, chat_record.content
            ));
        }

        for (room_id, room_record) in &lobby_state.rooms {
            ui.label(format!("room[{}]: {}", room_id, room_record.room_name));
            if ui.button("Enter").clicked() {
                action = Some(UiAction::EnterRoom(*room_id));
            }
        }

        if lobby_state.create_room_dialog_open {
            let mut should_close_dialog = false;
            egui::Window::new("Create Room")
                .collapsible(false)
                .resizable(false)
                .open(&mut lobby_state.create_room_dialog_open)
                .show(ui.ctx(), |ui| {
                    ui.label("RoomName");
                    ui.text_edit_singleline(&mut lobby_state.create_room_name_input);

                    let room_name = lobby_state.create_room_name_input.trim().to_string();
                    if ui
                        .add_enabled(!room_name.is_empty(), egui::Button::new("OK"))
                        .clicked()
                    {
                        action = Some(UiAction::CreateRoom(room_name));
                        lobby_state.create_room_name_input.clear();
                        should_close_dialog = true;
                    }
                });

            if should_close_dialog {
                lobby_state.create_room_dialog_open = false;
            }
        }

        return action;
    }

    fn handle_room(&mut self) {
        let ViewState::Room(room_state) = &mut self.state else {
            unreachable!("{:?}", self.state)
        };

        if let Ok(event) = self.rx_msg.try_recv() {
            match event {
                NetworkTaskEvent::Disconnected => {
                    self.change_state(ViewState::Home);
                }
                NetworkTaskEvent::Recv(msg) => match msg {
                    DownlinkMessage::RoomChat {
                        room_id,
                        client_id,
                        content,
                    } => {
                        assert!(room_id == room_state.room_id);
                        room_state.chats.push(ChatRecord { client_id, content });
                    }
                    _ => unreachable!("{:?}", msg),
                },
                _ => unreachable!("{:?}", event),
            }
        }
    }

    fn ui_room(&mut self, ui: &mut egui::Ui) -> Option<UiAction> {
        let ViewState::Room(room_state) = &mut self.state else {
            unreachable!()
        };

        let mut action: Option<UiAction> = None;
        ui.heading(format!("Room[{}]", room_state.room_id));
        ui.separator();
        ui.label(format!("client_id: {}", self.client_id.unwrap()));
        if ui.button("Disconnect").clicked() {
            action = Some(UiAction::Disconnect);
        }

        ui.label("Chat");

        let response = ui.text_edit_singleline(&mut room_state.chat_input);

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let content = room_state.chat_input.clone();
            room_state.chat_input.clear();

            action = Some(UiAction::SendRoomChat(content));
        }

        for chat_record in &room_state.chats {
            ui.label(format!(
                "client[{}]: {}",
                chat_record.client_id, chat_record.content
            ));
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
            ViewState::Room(_) => self.handle_room(),
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
                ViewState::Room(_) => self.ui_room(ui),
            }
        });

        // -------------------------
        // 3. Execute action
        // -------------------------

        // TODO 一堆乱七八糟的action在这里还要区分state才能处理？
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
                UiAction::CreateRoom(room_name) => {
                    let req_id = self.next_req("Create Room".to_string());
                    self.send(UplinkMessage::LobbyCreateRoom {
                        client_id: self.client_id.unwrap(),
                        req_id,
                        room_name,
                    });
                }
                UiAction::SendRoomChat(content) => {
                    let ViewState::Room(room_state) = &self.state else {
                        unreachable!()
                    };
                    self.send(UplinkMessage::RoomChat {
                        client_id: self.client_id.unwrap(),
                        room_id: room_state.room_id,
                        content,
                    });
                }
                UiAction::EnterRoom(room_id) => {
                    if !matches!(self.state, ViewState::Lobby(_)) {
                        unreachable!()
                    };
                    let req_id = self.next_req("Enter Room".to_string());
                    self.send(UplinkMessage::RoomEnter {
                        client_id: self.client_id.unwrap(),
                        req_id,
                        room_id,
                    });
                }
            }
        }

        ui.request_repaint();
    }
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::empty();

    fonts.font_data.insert(
        "my_font".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "LXGWWenKaiLite-Regular.ttf"
        ))),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    ctx.set_fonts(fonts);
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
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            Ok(Box::new(App::new(tx_cmd, rx_msg)))
        }),
    )
    .unwrap();
}
