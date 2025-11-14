use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Bullet {
    pub id: u32,
    pub owner_id: u32,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub speed: f32,
    pub range: f32,
    pub traveled: f32,
    pub damage: u32,
    pub hit_radius: f32,
}

impl Bullet {
    pub fn render(&self, offset_x: f32, offset_y: f32) {
        let draw_x = self.x * config::TILE_SIZE + offset_x;
        let draw_y = self.y * config::TILE_SIZE + offset_y;
        draw_circle(
            draw_x,
            draw_y,
            self.hit_radius * config::TILE_SIZE,
            DARKPURPLE,
        );
    }
}
