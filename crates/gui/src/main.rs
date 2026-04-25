use server::common::{ClientId, DownlinkMessage, ReqId, RoomId, UplinkMessage};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::TcpStream,
    sync::mpsc,
};

use eframe::egui;

#[derive(Debug)]
enum NetworkTaskCmd {
    Connect,
    Disconnect,
    Send(UplinkMessage),
}

#[derive(Debug)]
enum NetworkTaskEvent {
    Connected,
    Disconnected,
    Recv(DownlinkMessage),
}

async fn network_task(
    addr: String,
    mut rx_cmd: mpsc::UnboundedReceiver<NetworkTaskCmd>,
    tx_msg: mpsc::UnboundedSender<NetworkTaskEvent>,
) -> Result<(), std::io::Error> {
    use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

    #[derive(Debug)]
    enum State {
        Disconnected,
        Connected {
            writer: OwnedWriteHalf,
            lines: Lines<BufReader<OwnedReadHalf>>,
        },
    }

    let mut state = State::Disconnected;

    loop {
        match &mut state {
            State::Disconnected => {
                // Only react to commands
                while let Some(cmd) = rx_cmd.recv().await {
                    match cmd {
                        // GUI ask to connect
                        NetworkTaskCmd::Connect => match TcpStream::connect(&addr).await {
                            Ok(stream) => {
                                let (r, w) = stream.into_split();
                                state = State::Connected {
                                    writer: w,
                                    lines: BufReader::new(r).lines(),
                                };
                                tx_msg.send(NetworkTaskEvent::Connected).unwrap();
                                break;
                            }
                            Err(e) => {
                                eprintln!("connect failed: {e}");
                            }
                        },
                        _ => {
                            eprintln!("wrong event {:?} @ {:?}", cmd, state);
                        }
                    }
                }
            }

            State::Connected { lines, writer } => {
                tokio::select! {
                    // Commands
                    cmd = rx_cmd.recv() => {
                        match cmd {
                            Some(NetworkTaskCmd::Disconnect) => {
                                // GUI ask to disconnect
                                writer.shutdown().await.unwrap();
                                state = State::Disconnected;
                                tx_msg.send(NetworkTaskEvent::Disconnected).unwrap();
                            }
                            Some(NetworkTaskCmd::Send(msg)) => {
                                // GUI ask to send message
                                let json = serde_json::to_string(&msg)?;
                                writer.write_all(json.as_bytes()).await?;
                                writer.write_all(b"\n").await?;
                            }
                            Some(NetworkTaskCmd::Connect) => {
                                eprintln!("wrong event {:?} @ {:?}", cmd, state);
                            }
                            None => break,
                        }
                    }

                    // Socket read
                    result = lines.next_line() => {
                        match result {
                            Ok(Some(s)) => {
                                match serde_json::from_str::<DownlinkMessage>(&s) {
                                    Ok(msg) => {
                                        tx_msg.send(NetworkTaskEvent::Recv(msg)).unwrap();
                                    }
                                    Err(e) => {
                                        eprintln!("invalid json: {e}, line={s}");
                                    }
                                }
                            },
                            Ok(None) => {
                                eprintln!("server closed connection");
                                state = State::Disconnected;
                                tx_msg.send(NetworkTaskEvent::Disconnected).unwrap();
                                continue;
                            }
                            Err(e) => {
                                eprintln!("read error: {e}");
                                state = State::Disconnected;
                                tx_msg.send(NetworkTaskEvent::Disconnected).unwrap();
                                continue;
                            }
                        }

                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Default, Debug, Clone)]
struct LobbyState {
    pub chat_input: String,
}

#[derive(Debug, Clone)]
enum ViewState {
    Home,
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

    chat_log: Vec<String>,
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
            chat_log: vec![],
        }
    }

    fn next_req(&mut self) -> ReqId {
        let id = self.next_req_id;
        self.next_req_id += 1;
        id
    }

    fn send(&mut self, msg: UplinkMessage, desc: &str) {
        self.pending = Some(Pending {
            req_id: msg.req_id,
            description: desc.to_string(),
        });

        self.tx_cmd.send(NetworkTaskCmd::Send(msg)).unwrap();
    }

    fn connect(&mut self) {
        self.tx_cmd.send(NetworkTaskCmd::Connect).unwrap();
    }

    fn disconnect(&mut self) {
        self.tx_cmd.send(NetworkTaskCmd::Disconnect).unwrap();
    }

    fn handle_server_msg(&mut self, msg: DownlinkMessage) {
        let req_id = msg.req_id;
        match msg.msg {
            server::common::DownlinkMessageValue::Greeting(client_id) => {
                self.client_id = Some(client_id);
                let req_id = self.next_req();
                self.send(
                    UplinkMessage {
                        client_id,
                        req_id,
                        msg: server::common::UplinkMessageValue::Lobby(
                            server::common::UplinkLobbyMessage::Enter,
                        ),
                    },
                    "Enter Lobby",
                );
            }
            server::common::DownlinkMessageValue::PingEcho => todo!(),
            server::common::DownlinkMessageValue::Shutdown => todo!(),
            server::common::DownlinkMessageValue::Lobby(downlink_lobby_message) => {
                match downlink_lobby_message {
                    server::common::DownlinkLobbyMessage::EnterAck { success } => {
                        if self.pending_matches(req_id) {
                            self.pending = None;
                            if success {
                                self.state = ViewState::Lobby(LobbyState::default());
                            }
                        }
                    }
                    server::common::DownlinkLobbyMessage::Chat { client_id, content } => {
                        self.chat_log.push(format!("{client_id}: {content}"));
                    }
                }
            }
            server::common::DownlinkMessageValue::Room(downlink_room_message) => {
                match downlink_room_message {
                    server::common::DownlinkRoomMessage::CreateAck { success, room_id } => todo!(),
                    server::common::DownlinkRoomMessage::EnterAck { success, room_id } => {
                        if self.pending_matches(req_id) {
                            self.pending = None;
                            if success {
                                self.state = ViewState::Room(room_id);
                            }
                        }
                    }
                    server::common::DownlinkRoomMessage::Chat {
                        room_id,
                        client_id,
                        content,
                    } => todo!(),
                    server::common::DownlinkRoomMessage::QuitAck => todo!(),
                }
            }
        }
    }

    fn pending_matches(&self, req_id: ReqId) -> bool {
        self.pending
            .as_ref()
            .map(|p| p.req_id == req_id)
            .unwrap_or(false)
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // -------------------------
        // 1. Drain server messages
        // -------------------------
        while let Ok(msg) = self.rx_msg.try_recv() {
            match msg {
                NetworkTaskEvent::Connected => {}
                NetworkTaskEvent::Disconnected => {
                    self.state = ViewState::Home;
                }
                NetworkTaskEvent::Recv(downlink_message) => {
                    self.handle_server_msg(downlink_message);
                }
            }
        }

        // -------------------------
        // 2. UI → collect action
        // -------------------------
        let mut action: Option<UiAction> = None;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(p) = &self.pending {
                ui.label(format!("Pending: {}", p.description));
            }

            match &mut self.state {
                ViewState::Home => {
                    ui.heading("Home");
                    ui.separator();

                    if ui.button("Connect").clicked() {
                        action = Some(UiAction::Connect);
                    }
                }

                ViewState::Lobby(lobby_state) => {
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

                    for line in &self.chat_log {
                        ui.label(line);
                    }
                }

                ViewState::Room(id) => {
                    ui.label(format!("You are in Room {}", id));

                    // future action:
                    // if ui.button("Leave").clicked() {
                    //     action.push(UiAction::LeaveRoom);
                    // }
                }
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
                    let req_id = self.next_req();

                    self.send(
                        UplinkMessage {
                            client_id: self.client_id.unwrap(),
                            req_id,
                            msg: server::common::UplinkMessageValue::Lobby(
                                server::common::UplinkLobbyMessage::Chat { content },
                            ),
                        },
                        "Send LobbyChat",
                    );
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
