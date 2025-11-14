use crate::config;
use ::rand::Rng;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModifierKind {
    Heal(u32),
    Speed(f32),
    Damage(u32),
    FireRate(f32),
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
            ModifierKind::FireRate(_) => LIME,
        };
        draw_circle(draw_x, draw_y, config::TILE_SIZE * 0.5, color);
    }
}

impl ModifierKind {
    pub fn random(rng: &mut impl Rng) -> Self {
        let choice = rng.gen_range(0..4);
        match choice {
            0 => ModifierKind::Heal(1),
            1 => ModifierKind::Speed(1.0),
            2 => ModifierKind::Damage(1),
            _ => ModifierKind::FireRate(0.5),
        }
    }
}
