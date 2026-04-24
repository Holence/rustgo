use server::common::{ClientId, DownlinkMessage, ReqId, RoomId, UplinkMessage};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::TcpStream,
    spawn,
    sync::mpsc,
};

use eframe::egui;

#[derive(Debug)]
enum NetworkTaskCmd {
    Connect,
    Send(UplinkMessage),
    Disconnect,
}

pub async fn network_task(
    addr: String,
    mut rx_cmd: mpsc::UnboundedReceiver<NetworkTaskCmd>,
    tx_msg: mpsc::UnboundedSender<DownlinkMessage>,
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
                        NetworkTaskCmd::Connect => match TcpStream::connect(&addr).await {
                            Ok(stream) => {
                                let (r, w) = stream.into_split();
                                state = State::Connected {
                                    writer: w,
                                    lines: BufReader::new(r).lines(),
                                };
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
                                state = State::Disconnected;
                            }
                            Some(NetworkTaskCmd::Send(msg)) => {
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
                                if let Ok(msg) = serde_json::from_str::<DownlinkMessage>(&s) {
                                    tx_msg.send(msg).unwrap();
                                }
                            },
                            _ => {
                                state = State::Disconnected;
                                continue;
                            },
                        }

                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum ViewState {
    Home,
    Lobby,
    Room(RoomId),
}

#[derive(Debug)]
struct Pending {
    req_id: ReqId,
    description: String,
}

pub struct App {
    state: ViewState,
    pending: Option<Pending>,
    next_req_id: ReqId,
    client_id: Option<ClientId>,

    tx_cmd: mpsc::UnboundedSender<NetworkTaskCmd>,
    rx_msg: mpsc::UnboundedReceiver<DownlinkMessage>,

    chat_log: Vec<String>,
}

impl App {
    pub fn new(
        tx_cmd: mpsc::UnboundedSender<NetworkTaskCmd>,
        rx_msg: mpsc::UnboundedReceiver<DownlinkMessage>,
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
                                self.state = ViewState::Lobby;
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
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // 1. Drain server messages
        while let Ok(msg) = self.rx_msg.try_recv() {
            self.handle_server_msg(msg);
        }

        // 2. UI
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Client");

            if let Some(p) = &self.pending {
                ui.label(format!("Pending: {}", p.description));
            }

            match self.state {
                ViewState::Home => {
                    ui.heading("Home");
                    if ui.button("Connect").clicked() {
                        self.connect();
                    }
                }
                ViewState::Lobby => {
                    ui.heading("Lobby");
                    ui.separator();
                    ui.heading("Chat");

                    for line in &self.chat_log {
                        ui.label(line);
                    }
                }

                ViewState::Room(id) => {
                    ui.label(format!("You are in Room {}", id));
                }
            }
        });

        ui.request_repaint(); // keep UI responsive
    }
}

#[tokio::main]
async fn main() {
    let (tx_cmd, rx_cmd) = mpsc::unbounded_channel();
    let (tx_msg, rx_msg) = mpsc::unbounded_channel();

    spawn(network_task("127.0.0.1:8080".to_string(), rx_cmd, tx_msg));

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Client",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(tx_cmd, rx_msg)))),
    )
    .unwrap();
}
