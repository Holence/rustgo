use server::common::{DownlinkMessage, UplinkMessage};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::TcpStream,
    sync::mpsc,
};

#[derive(Debug)]
pub enum NetworkTaskCmd {
    Connect,
    Disconnect,
    Send(UplinkMessage),
}

#[derive(Debug)]
pub enum NetworkTaskEvent {
    Connected,
    Disconnected,
    Recv(DownlinkMessage),
}

pub async fn network_task(
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
                                let (reader, writer) = stream.into_split();
                                state = State::Connected {
                                    writer,
                                    lines: BufReader::new(reader).lines(),
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
                    biased;

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
