use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub id: u32,
    pub x: usize,
    pub y: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub players: HashMap<u32, Player>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.players.keys().max().map(|id| id + 1).unwrap_or(1)
    }

    pub fn remove(&mut self, id: u32) {
        self.players.remove(&id);
    }

    pub fn create_player(&self) -> Player {
        let id = self.next_id();
        let x = 1;
        let y = 1;
        Player { id, x, y }
    }
}
