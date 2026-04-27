use log::info;
use server::{lobby::LobbyActor, router::RouterActor, session::SessionActor};
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

    let (router_tx, router_rx) = mpsc::channel(128);
    let (lobby_tx, lobby_rx) = mpsc::channel(32);

    let router_actor = RouterActor::new(router_rx, lobby_tx);
    tokio::spawn(router_actor.run());

    let lobby_actor = LobbyActor::new(lobby_rx, router_tx.clone());
    tokio::spawn(lobby_actor.run());

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        info!("{} connected", addr);
        let session_actor = SessionActor::new(stream);
        tokio::spawn(session_actor.run(router_tx.clone()));
    }
    // ctrl+c drop(router_tx);
}
