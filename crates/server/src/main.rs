use log::info;
use server::{client::ClientActor, lobby::LobbyActor};
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            let style = buf.default_level_style(record.level());

            use std::io::Write;
            writeln!(
                buf,
                "[{} {style}{:<5}{style:#} {}:{}] {}",
                ts,
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                record.args()
            )
        })
        .init();

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("Server running on 8080");

    let (lobby_tx, lobby_rx) = mpsc::channel(32);
    let lobby_actor = LobbyActor::new(lobby_rx);
    tokio::spawn(lobby_actor.run());

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        info!("{} connected", addr);
        let client_actor = ClientActor::new(stream, addr, lobby_tx.clone());
        tokio::spawn(client_actor.run());
    }
    // ctrl+c drop(lobby_tx);
}
