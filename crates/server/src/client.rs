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
    common::{ClientId, UplinkMessage},
    lobby::LobbyMessage,
};

pub struct ClientActor {
    addr: SocketAddr,
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    client_id: ClientId,
}

impl ClientActor {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        let (reader, writer) = stream.into_split();
        let reader: BufReader<OwnedReadHalf> = BufReader::new(reader);
        ClientActor {
            addr,
            reader,
            writer,
            client_id: 0,
        }
    }

    pub async fn run(mut self, lobby_tx: mpsc::Sender<LobbyMessage>) {
        let mut lines = self.reader.lines();
        let Some(line) = lines.next_line().await.unwrap() else {
            error!("{} is bad", self.addr);
            return;
        };

        // wait for Login message
        let Ok(UplinkMessage::Login { username }) = serde_json::from_str::<UplinkMessage>(&line)
        else {
            error!(
                "{} first packet is not UplinkMessage::Login, received: '{}'",
                self.addr, line
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
        sleep(Duration::from_millis(1000)).await;
        let Some(client_id) = client_id_rx.await.unwrap() else {
            // login failed
            error!("{} login failed", self.addr);
            let msg = serde_json::to_string(&crate::common::DownlinkMessage::LoginAck {
                client_id: None,
            })
            .unwrap();
            self.writer.write_all(msg.as_bytes()).await.unwrap();
            self.writer.write_all(b"\n").await.unwrap();
            self.writer.shutdown().await.unwrap();
            info!("client[{}] writer shutdown", self.client_id);
            return;
        };

        // login success
        self.client_id = client_id;
        let msg = serde_json::to_string(&crate::common::DownlinkMessage::LoginAck {
            client_id: Some(client_id),
        })
        .unwrap();
        self.writer.write_all(msg.as_bytes()).await.unwrap();
        self.writer.write_all(b"\n").await.unwrap();
        info!("client[{}] conncted", self.client_id);

        // writer task
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = client_rx.recv().await {
                let msg = serde_json::to_string(&msg).unwrap();
                info!("client[{}] write {msg}", self.client_id);
                self.writer.write_all(msg.as_bytes()).await.unwrap();
                self.writer.write_all(b"\n").await.unwrap();
            }
            self.writer.shutdown().await.unwrap();
            info!("client[{}] writer shutdown", self.client_id);
        });

        // reader loop
        while let Ok(Some(line)) = lines.next_line().await {
            info!("[{}] read {line}", self.client_id);
            match serde_json::from_str::<UplinkMessage>(&line) {
                Ok(msg) => {
                    if matches!(msg, UplinkMessage::Login { .. }) {
                        error!("duplicate login message from client[{}]", self.client_id);
                        continue;
                    }
                    // TODO some check and filter for malicious package?
                    // client_id must equal to self.client_id
                    lobby_tx
                        .send(LobbyMessage::ClientMessage { msg })
                        .await
                        .unwrap();
                }
                Err(err) => {
                    error!("client[{}] parse uplink error: {err}", self.client_id);
                }
            }
        }
        info!("client[{}] reader stop", self.client_id);

        lobby_tx
            .send(LobbyMessage::UnregisterClient {
                client_id: self.client_id,
            })
            .await
            .unwrap();

        writer_task.await.unwrap();

        info!("client[{}] disconncted", self.client_id);
    }
}
