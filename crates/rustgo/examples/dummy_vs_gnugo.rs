use rustgo::{
    Stone,
    game::{Game, Team},
    player::{dummy_player::DummyPlayer, local_gnugo_player::LocalGnugoPlayer},
};

const BOARD_SIZE: usize = 13;
fn main() {
    let team1 = Team::new(
        Stone::BLACK,
        vec![
            Box::new(DummyPlayer::new(BOARD_SIZE)),
            Box::new(DummyPlayer::new(BOARD_SIZE)),
            Box::new(DummyPlayer::new(BOARD_SIZE)),
            Box::new(LocalGnugoPlayer::new(BOARD_SIZE).unwrap()),
        ],
    );
    let team2 = Team::new(
        Stone::WHITE,
        vec![
            Box::new(LocalGnugoPlayer::new(BOARD_SIZE).unwrap()),
            Box::new(LocalGnugoPlayer::new(BOARD_SIZE).unwrap()),
            Box::new(LocalGnugoPlayer::new(BOARD_SIZE).unwrap()),
            Box::new(DummyPlayer::new(BOARD_SIZE)),
        ],
    );
    let mut game = Game::new(BOARD_SIZE, vec![team1, team2]);
    game.run();
}
