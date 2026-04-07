use game::game::GameBuilder;
use game::player::PlayerId;
use game::player::dummy_player::DummyPlayer;
use game::player::local_gnugo_player::LocalGnugoPlayer;
use game::team::TeamId;
use rustgo::Stone;

const BOARD_SIZE: usize = 19;

#[tokio::main]
async fn main() {
    let mut game = GameBuilder::new(BOARD_SIZE);
    game.add_team(TeamId::new(0), Stone::BLACK);
    game.add_player(
        TeamId::new(0),
        PlayerId::new(0),
        "Dummy0".to_string(),
        DummyPlayer::new(BOARD_SIZE),
    );
    game.add_player(
        TeamId::new(0),
        PlayerId::new(1),
        "Dummy1".to_string(),
        DummyPlayer::new(BOARD_SIZE),
    );

    game.add_team(TeamId::new(10), Stone::WHITE);
    game.add_player(
        TeamId::new(10),
        PlayerId::new(10),
        "GnuGo0".to_string(),
        LocalGnugoPlayer::new(BOARD_SIZE).unwrap(),
    );
    game.add_player(
        TeamId::new(10),
        PlayerId::new(11),
        "GnuGo1".to_string(),
        LocalGnugoPlayer::new(BOARD_SIZE).unwrap(),
    );

    let mut game = game.build();
    game.run().await;
}
