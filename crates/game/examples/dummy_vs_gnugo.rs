use game::game::Game;
use game::player::dummy_player::DummyPlayer;
use game::player::local_gnugo_player::LocalGnugoPlayer;
use game::player::{PlayerId, PlayerTrait};
use game::team::{TeamBuilder, TeamHandle, TeamId};
use rustgo::Stone;
use tokio::sync::mpsc;

const BOARD_SIZE: usize = 19;

#[tokio::main]
async fn main() {
    let (uplink_tx, uplink_rx) = mpsc::channel(1024);

    let mut team_handles = vec![];
    let mut team_build = TeamBuilder::new();

    let mut player_handles = vec![];
    team_build.add_team(TeamId::new(0), Stone::BLACK).unwrap();
    team_build
        .add_player(TeamId::new(0), PlayerId::new(0), "Dummy0".to_string())
        .unwrap();
    let p = DummyPlayer::new(BOARD_SIZE);
    player_handles.push(p.spawn(PlayerId::new(0), uplink_tx.clone()));
    team_handles.push(TeamHandle::new(
        TeamId::new(0),
        Stone::BLACK,
        player_handles,
    ));

    let mut player_handles = vec![];
    team_build.add_team(TeamId::new(1), Stone::WHITE).unwrap();
    team_build
        .add_player(TeamId::new(1), PlayerId::new(10), "GnuGo0".to_string())
        .unwrap();
    let p = LocalGnugoPlayer::new(BOARD_SIZE).unwrap();
    player_handles.push(p.spawn(PlayerId::new(10), uplink_tx.clone()));
    team_handles.push(TeamHandle::new(
        TeamId::new(1),
        Stone::WHITE,
        player_handles,
    ));

    drop(uplink_tx);

    let team_infos = team_build.take();

    let mut game = Game::new(BOARD_SIZE, uplink_rx, team_infos, team_handles);
    game.run().await;
}
