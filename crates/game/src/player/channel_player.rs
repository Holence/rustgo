use std::sync::mpsc::{Receiver, Sender};

use rustgo::Stone;

use crate::player::{GameMessage, MoveAction, PlayerError, PlayerTrait};

// 用队列连接的Player，仅用于对接GUI前端
pub struct ChannelPlayer {
    tx: Sender<GameMessage>,
    rx: Receiver<MoveAction>,
}

impl ChannelPlayer {
    pub fn new(tx: Sender<GameMessage>, rx: Receiver<MoveAction>) -> Self {
        ChannelPlayer { tx, rx }
    }
}

impl PlayerTrait for ChannelPlayer {
    fn play(&mut self, move_action: MoveAction) -> Result<(), PlayerError> {
        self.tx.send(GameMessage::MoveAction(move_action)).unwrap(); // TODO handle error
        Ok(())
    }

    fn genmove(&mut self, stone: Stone) -> Result<MoveAction, PlayerError> {
        self.tx.send(GameMessage::GenMove(stone)).unwrap(); // TODO handle error
        let move_action = self.rx.recv().unwrap(); // TODO handle error
        return Ok(move_action);
    }
}
