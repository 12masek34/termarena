use crate::config;
use ::rand::Rng;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModifierKind {
    Heal(u32),
    Speed(f32),
    Damage(u32),
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
        let color = match self.kind {
            ModifierKind::Heal(_) => RED,
            ModifierKind::Damage(_) => MAGENTA,
            ModifierKind::Speed(_) => GREEN,
        };
        draw_circle(draw_x, draw_y, config::TILE_SIZE * 0.5, color);
    }
}

impl ModifierKind {
    pub fn random(rng: &mut impl Rng) -> Self {
        let roll = rng.gen_range(0..100);

        if roll < 25 {
            ModifierKind::Heal(1)
        } else if roll < 85 {
            ModifierKind::Speed(0.1)
        } else {
            ModifierKind::Damage(1)
        }
    }
}
