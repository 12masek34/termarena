use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::map::Map;
use rand::Rng;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u32,
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub fn create_player(&mut self, map: &Map) -> Player {
        let mut rng = rand::thread_rng();
        let id = self.next_id();
        let x = rng.gen_range(0..map.width);
        let y = rng.gen_range(0..map.height);
        let player = Player { id, x, y };
        self.players.insert(id, player.clone());

        player
    }
}
