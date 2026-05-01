use std::time::Duration;

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
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    client_id: ClientId,
}

impl ClientActor {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        let reader: BufReader<OwnedReadHalf> = BufReader::new(reader);
        ClientActor {
            reader,
            writer,
            client_id: 0,
        }
    }

    pub async fn run(mut self, lobby_tx: mpsc::Sender<LobbyMessage>) {
        let mut lines = self.reader.lines();

        let (client_tx, mut client_rx) = mpsc::channel(32);

        let (client_id_tx, client_id_rx) = oneshot::channel::<ClientId>();
        lobby_tx
            .send(LobbyMessage::RegisterClient {
                client_id_tx,
                client_tx,
            })
            .await
            .unwrap();

        sleep(Duration::from_millis(500)).await;
        self.client_id = client_id_rx.await.unwrap();
        info!("client_id[{}] conncted", self.client_id);

        // writer task
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = client_rx.recv().await {
                let msg = serde_json::to_string(&msg).unwrap();
                info!("client[{}] write {msg}", self.client_id);
                self.writer.write_all(msg.as_bytes()).await.unwrap();
                self.writer.write_all(b"\n").await.unwrap();
            }
        });

        // reader loop
        while let Ok(Some(line)) = lines.next_line().await {
            info!("[{}] read {line}", self.client_id);
            match serde_json::from_str::<UplinkMessage>(&line) {
                Ok(msg) => {
                    // TODO some check and filter for malicious package?
                    lobby_tx
                        .send(LobbyMessage::ClientMessage { msg })
                        .await
                        .unwrap();
                }
                Err(err) => {
                    error!("[{}] parse uplink error: {err}", self.client_id);
                }
            }
        }

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
