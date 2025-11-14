use crate::config;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModifierKind {
    Heal(u32),
    SpeedBoost(f32),
    DamageBoost(f32),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Modifier {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub kind: ModifierKind,
}

impl Modifier {
    pub fn render(&self, offset_x: f32, offset_y: f32) {
        let draw_x = self.x * config::TILE_SIZE + offset_x;
        let draw_y = self.y * config::TILE_SIZE + offset_y;
        draw_circle(draw_x, draw_y, config::TILE_SIZE * 0.5, RED);
    }
}
