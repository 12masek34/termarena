use std::sync::{Arc, Mutex};

use crate::{
    game::{player::Player, state::GameState},
    map::Map,
};

#[derive(Debug)]
pub struct ClientState {
    pub id: Option<u32>,
    pub map: Option<Arc<Mutex<Map>>>,
    pub game_state: Option<Arc<GameState>>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            id: None,
            map: None,
            game_state: None,
        }
    }

    pub fn init_player(&mut self, player: Player) {
        if self.id.is_none() {
            self.id = Some(player.id);
        }
    }

    pub fn update_state(&mut self, state: GameState) {
        self.game_state = Some(Arc::new(state));
    }

    pub fn get_current_player(&self) -> Option<Player> {
        if let Some(gs) = &self.game_state {
            self.id.and_then(|id| gs.players.get(&id).cloned())
        } else {
            None
        }
    }
}
