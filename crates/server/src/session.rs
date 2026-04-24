use log::{error, info, log};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{mpsc, oneshot},
};

use crate::{
    common::{ClientId, DownlinkMessage, UplinkMessage},
    router::RouterMessage,
};

pub type SessionActorRx = mpsc::Receiver<UplinkMessage>;
pub type SessionActorTx = mpsc::Sender<DownlinkMessage>;

pub struct SessionActor {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    client_id: ClientId,
}

impl SessionActor {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        let reader: BufReader<OwnedReadHalf> = BufReader::new(reader);
        SessionActor {
            reader,
            writer,
            client_id: 0,
        }
    }

    pub async fn run(mut self, router_tx: mpsc::Sender<RouterMessage>) {
        let mut lines = self.reader.lines();

        let (session_tx, mut session_rx) = mpsc::channel(32);

        let (client_id_tx, client_id_rx) = oneshot::channel::<ClientId>();
        router_tx
            .send(RouterMessage::RegisterSession {
                client_id_tx,
                session_tx,
            })
            .await
            .unwrap();

        self.client_id = client_id_rx.await.unwrap();

        // writer task
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = session_rx.recv().await {
                let msg = serde_json::to_string(&msg).unwrap();
                info!("[{}] write {msg}", self.client_id);
                self.writer.write_all(msg.as_bytes()).await.unwrap();
                self.writer.write_all(b"\n").await.unwrap();
            }
        });

        // reader loop
        while let Ok(Some(line)) = lines.next_line().await {
            info!("[{}] read {line}", self.client_id);
            match serde_json::from_str::<UplinkMessage>(&line) {
                Ok(msg) => {
                    if msg.client_id != self.client_id {
                        error!(
                            "[{}] client_id does not match: {}",
                            self.client_id, msg.client_id
                        );
                    } else {
                        router_tx
                            .send(RouterMessage::ClientMessage { msg })
                            .await
                            .unwrap();
                    }
                }
                Err(err) => {
                    eprintln!("[{}] parse uplink error: {err}", self.client_id);
                }
            }
        }

        router_tx
            .send(RouterMessage::UnregisterSession {
                client_id: self.client_id,
            })
            .await
            .unwrap();

        writer_task.await.unwrap();

        println!("Session {} stopped", self.client_id);
    }
}
