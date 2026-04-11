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
        DummyPlayer::new(PlayerId::new(0), BOARD_SIZE),
    );
    game.add_player(
        TeamId::new(0),
        DummyPlayer::new(PlayerId::new(1), BOARD_SIZE),
    );

    game.add_team(TeamId::new(10), Stone::WHITE);
    game.add_player(
        TeamId::new(10),
        LocalGnugoPlayer::new(PlayerId::new(10), BOARD_SIZE).unwrap(),
    );
    game.add_player(
        TeamId::new(10),
        LocalGnugoPlayer::new(PlayerId::new(11), BOARD_SIZE).unwrap(),
    );

    let mut game = game.build();
    game.run().await;
}
