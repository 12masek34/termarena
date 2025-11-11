use crate::game::state::{GameState, Player};

#[derive(Debug)]
pub struct ClientState {
    pub id: Option<u32>,
    pub game_state: Option<GameState>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            id: None,
            game_state: None,
        }
    }

    pub fn init_player(&mut self, player: Player) {
        self.id = Some(player.id);
    }

    pub fn update_state(&mut self, state: GameState) {
        self.game_state = Some(state);
    }

    pub fn get_current_player(&self) -> Option<Player> {
        if let Some(gs) = &self.game_state {
            self.id.and_then(|id| gs.players.get(&id).cloned())
        } else {
            None
        }
    }
}
