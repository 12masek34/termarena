use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::map::Map;
use rand::Rng;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

    pub fn create_map(&mut self) {}

    pub fn create_player(&mut self, map: &Map) -> Player {
        let id = self.next_id();
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..map.width) as f32;
        let y = rng.gen_range(0..map.height) as f32;
        let player = Player { id, x, y };
        self.players.insert(id, player.clone());

        player
    }

    pub fn move_player(&mut self, player_id: Option<u32>, x: f32, y: f32) {
        if let Some(id) = player_id {
            if let Some(player) = self.players.get_mut(&id) {
                player.x += x;
                player.y += y;
            }
        }
    }
}
