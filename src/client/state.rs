use std::sync::{Arc, Mutex};

use crate::{
    game::{
        player::Player,
        state::{GameState, GameStateDiff},
    },
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

    pub fn update_state_diff(&mut self, state_diff: GameStateDiff) {
        if let Some(gs_arc) = &self.game_state {
            let mut gs_arc_clone = Arc::clone(gs_arc);
            let gs = Arc::make_mut(&mut gs_arc_clone);

            for (id, player) in state_diff.players {
                gs.players.insert(id, player);
            }
            for id in state_diff.removed_players {
                if id == self.id.unwrap() {
                    continue;
                }
                if let Some(player) = gs.players.get_mut(&id) {
                    player.to_render = false;
                }
            }

            for (id, bullet) in state_diff.bullets {
                gs.bullets.insert(id, bullet);
            }
            for id in state_diff.removed_bullets {
                gs.bullets.remove(&id);
            }

            for (id, modifier) in state_diff.modifieres {
                gs.modifieres.insert(id, modifier);
            }
            for id in state_diff.removed_modifieres {
                gs.modifieres.remove(&id);
            }

            self.game_state = Some(gs_arc_clone);
        } else {
            let mut gs = GameState::new();

            for (id, player) in state_diff.players {
                gs.players.insert(id, player);
            }
            for (id, bullet) in state_diff.bullets {
                gs.bullets.insert(id, bullet);
            }
            for (id, modifier) in state_diff.modifieres {
                gs.modifieres.insert(id, modifier);
            }

            self.game_state = Some(Arc::new(gs));
        }
    }

    pub fn get_current_player(&self) -> Option<Player> {
        if let Some(gs) = &self.game_state {
            self.id.and_then(|id| gs.players.get(&id).cloned())
        } else {
            None
        }
    }
}
