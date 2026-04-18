use server::{common::SessionId, lobby::LobbyActor, router::RouterActor, session::SessionActor};
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on 8080");

    let (lobby_tx, lobby_rx) = mpsc::channel(32);
    let lobby_actor = LobbyActor::new(lobby_rx);
    tokio::spawn(lobby_actor.run());

    let (router_tx, router_rx) = mpsc::channel(128);
    let router_actor = RouterActor::new(router_rx, lobby_tx);
    tokio::spawn(router_actor.run());

    let mut session_id: SessionId = 0;
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let session_actor = SessionActor::new(stream);
        tokio::spawn(session_actor.run(router_tx.clone(), session_id));
        session_id += 1;
    }
}
