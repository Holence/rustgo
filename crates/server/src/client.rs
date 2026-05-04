use std::{net::SocketAddr, time::Duration};

use log::{error, info};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::{
    common::{ClientId, DownlinkMessage, UplinkMessage},
    lobby::LobbyMessage,
};

pub struct ClientActor {
    addr: SocketAddr,
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    lobby_tx: mpsc::Sender<LobbyMessage>,
}

macro_rules! write_line {
    ($writer:expr, $msg:expr) => {
        $writer
            .write_all(serde_json::to_string(&$msg).unwrap().as_bytes())
            .await
            .unwrap();
        $writer.write_all(b"\n").await.unwrap();
    };
}

impl ClientActor {
    pub fn new(stream: TcpStream, addr: SocketAddr, lobby_tx: mpsc::Sender<LobbyMessage>) -> Self {
        let (reader, writer) = stream.into_split();
        let reader: BufReader<OwnedReadHalf> = BufReader::new(reader);
        ClientActor {
            addr,
            reader,
            writer,
            lobby_tx,
        }
    }

    pub async fn run(self) {
        let Self {
            addr,
            reader,
            mut writer,
            lobby_tx,
        } = self;

        let mut lines = reader.lines();
        let Some(line) = lines.next_line().await.unwrap() else {
            error!("{} is bad", addr);
            return;
        };

        // wait for Login message
        let Ok(UplinkMessage::Login { username }) = serde_json::from_str::<UplinkMessage>(&line)
        else {
            error!(
                "{} first packet is not UplinkMessage::Login, received: '{}'",
                addr, line
            );
            return;
        };

        // send username to Lobby
        let (client_tx, mut client_rx) = mpsc::channel(32);
        let (client_id_tx, client_id_rx) = oneshot::channel::<Option<ClientId>>();
        lobby_tx
            .send(LobbyMessage::RegisterClient {
                username,
                client_id_tx,
                client_tx,
            })
            .await
            .unwrap();

        // wait for confirm
        sleep(Duration::from_millis(500)).await;
        let Some(client_id) = client_id_rx.await.unwrap() else {
            // login failed
            error!("{} login failed", addr);
            write_line!(writer, DownlinkMessage::LoginAck { client_id: None });
            writer.shutdown().await.unwrap();
            info!("{} writer shutdown", addr);
            return;
        };

        // login success
        write_line!(
            writer,
            DownlinkMessage::LoginAck {
                client_id: Some(client_id),
            }
        );
        info!("client[{}] conncted", client_id);

        // writer task
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = client_rx.recv().await {
                info!("client[{}] write {:?}", client_id, msg);
                write_line!(writer, msg);
            }
            writer.shutdown().await.unwrap();
            info!("client[{}] writer shutdown", client_id);
        });

        // reader loop
        while let Ok(Some(line)) = lines.next_line().await {
            info!("client[{}] read {line}", client_id);
            match serde_json::from_str::<UplinkMessage>(&line) {
                Ok(msg) => {
                    if matches!(msg, UplinkMessage::Login { .. }) {
                        error!("duplicate login message from client[{}]", client_id);
                        continue;
                    }
                    // TODO some check and filter for malicious package?
                    // client_id must equal to client_id
                    lobby_tx
                        .send(LobbyMessage::ClientMessage { msg })
                        .await
                        .unwrap();
                }
                Err(err) => {
                    error!("client[{}] parse uplink error: {err}", client_id);
                }
            }
        }
        info!("client[{}] reader stop", client_id);

        lobby_tx
            .send(LobbyMessage::UnregisterClient { client_id })
            .await
            .unwrap();

        writer_task.await.unwrap();

        info!("client[{}] disconncted", client_id);
    }
}
