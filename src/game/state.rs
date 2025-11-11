use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::map::Map;
use ::rand::Rng;
use ::rand::thread_rng;

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
        let mut rng = thread_rng();
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

    pub fn render(&self, current_id: Option<u32>, player_pos: (f32, f32)) {
        let tile_size = 10.0;
        let offset_x = screen_width() / 2.0 - player_pos.0 * tile_size;
        let offset_y = screen_height() / 2.0 - player_pos.1 * tile_size;

        for player in self.players.values() {
            let draw_x = player.x * tile_size + offset_x;
            let draw_y = player.y * tile_size + offset_y;
            let color = if Some(player.id) == current_id {
                BLUE
            } else {
                RED
            };
            draw_circle(draw_x, draw_y, tile_size, color);
        }
    }
}
