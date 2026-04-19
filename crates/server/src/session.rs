use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc,
};

use crate::{
    common::{DownlinkMessage, SessionId, UplinkMessage},
    router::RouterMessage,
};

pub type SessionActorRx = mpsc::Receiver<UplinkMessage>;
pub type SessionActorTx = mpsc::Sender<DownlinkMessage>;

pub struct SessionActor {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl SessionActor {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        let reader: BufReader<OwnedReadHalf> = BufReader::new(reader);
        SessionActor { reader, writer }
    }

    pub async fn run(mut self, router_tx: mpsc::Sender<RouterMessage>, session_id: SessionId) {
        println!("Session {} started", session_id);
        let mut lines = self.reader.lines();

        let (tx, mut rx) = mpsc::channel(32);

        router_tx
            .send(RouterMessage::RegisterSession {
                session_id,
                session_tx: tx,
            })
            .await
            .unwrap();

        // writer task
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let msg = serde_json::to_string(&msg).unwrap();
                self.writer.write_all(msg.as_bytes()).await.unwrap();
            }
        });

        // reader loop
        while let Ok(Some(line)) = lines.next_line().await {
            match serde_json::from_str::<UplinkMessage>(&line) {
                Ok(msg) => {
                    let _ = router_tx
                        .send(RouterMessage::ClientMessage { session_id, msg })
                        .await;
                }
                Err(err) => {
                    eprintln!("Session {session_id} parse uplink error: {err}");
                }
            }
        }

        router_tx
            .send(RouterMessage::UnregisterSession { session_id })
            .await
            .unwrap();

        writer_task.await.unwrap();

        println!("Session {} stopped", session_id);
    }
}
