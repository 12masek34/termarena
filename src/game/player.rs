use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::config;

use super::{bullet::Bullet, state::Direction};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub kills: u32,
    pub deths: u32,
    pub radius: f32,
    pub direction: Direction,
    pub fire_rate: f32,
    pub bullet_speed: f32,
    pub bullet_range: f32,
    pub bullet_damage: u32,
    pub health: u32,
    pub max_health: u32,
    pub hit_radius: f32,
    pub is_moving: bool,
    pub move_target: Option<(f32, f32)>,
    pub walk_speed: f32,
    pub to_render: bool,

    #[serde(skip, default = "Player::default_last_shot")]
    pub last_shot: Instant,
}

impl Player {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            kills: 0,
            deths: 0,
            radius: config::PLAYER_RADIUS,
            direction: Direction::Up,
            last_shot: Instant::now() - Duration::from_secs(5),
            fire_rate: config::FIRE_RATE,
            bullet_speed: config::BULLET_SPEED,
            bullet_range: config::BULLET_RANGE,
            bullet_damage: config::BULLET_DAMAGE,
            health: config::PLAYER_HEALTH,
            max_health: config::PLAYER_HEALTH,
            hit_radius: config::HIT_RADIUS,
            is_moving: false,
            move_target: None,
            walk_speed: config::WALK_SPEED,
            to_render: true,
        }
    }

    fn default_last_shot() -> Instant {
        Instant::now() - Duration::from_secs(5)
    }

    pub fn hit_by(&mut self, bullet: &Bullet) -> bool {
        let dx = bullet.x - self.x;
        let dy = bullet.y - self.y;
        let distance_sq = dx * dx + dy * dy;
        let hit_distance = self.radius + bullet.hit_radius;

        if distance_sq < hit_distance * hit_distance {
            self.health = self.health.saturating_sub(bullet.damage);
            true
        } else {
            false
        }
    }

    pub fn render(&self, current_id: Option<u32>, offset_x: f32, offset_y: f32) {
        if !self.to_render {
            return;
        }
        let draw_x = self.x * config::TILE_SIZE + offset_x;
        let draw_y = self.y * config::TILE_SIZE + offset_y;

        let color = if Some(self.id) == current_id {
            BLUE
        } else {
            DARKBLUE
        };
        draw_circle(draw_x, draw_y, self.radius * config::TILE_SIZE, color);

        let tip_length = self.radius * config::TILE_SIZE * 1.5;
        let tip_width = self.radius * config::TILE_SIZE * 2.0;

        let (tip, left, right) = match self.direction {
            Direction::Up => (
                vec2(draw_x, draw_y - tip_length),
                vec2(draw_x - tip_width / 2.0, draw_y),
                vec2(draw_x + tip_width / 2.0, draw_y),
            ),
            Direction::Down => (
                vec2(draw_x, draw_y + tip_length),
                vec2(draw_x - tip_width / 2.0, draw_y),
                vec2(draw_x + tip_width / 2.0, draw_y),
            ),
            Direction::Left => (
                vec2(draw_x - tip_length, draw_y),
                vec2(draw_x, draw_y - tip_width / 2.0),
                vec2(draw_x, draw_y + tip_width / 2.0),
            ),
            Direction::Right => (
                vec2(draw_x + tip_length, draw_y),
                vec2(draw_x, draw_y - tip_width / 2.0),
                vec2(draw_x, draw_y + tip_width / 2.0),
            ),
        };

        draw_triangle(tip, left, right, color);

        let bar_width = config::TILE_SIZE * 2.0;
        let bar_height = 4.0;
        let health_ratio = self.health as f32 / self.max_health as f32;
        let bar_width_coef = 2.0;
        let draw_y_coef = 0.6;

        draw_rectangle(
            draw_x - bar_width / bar_width_coef,
            draw_y + config::TILE_SIZE / draw_y_coef,
            bar_width,
            bar_height,
            RED,
        );

        draw_rectangle(
            draw_x - bar_width / bar_width_coef,
            draw_y + config::TILE_SIZE / draw_y_coef,
            bar_width * health_ratio,
            bar_height,
            GREEN,
        );

        let text = format!("{}", self.id);
        let font_size = (self.radius * config::TILE_SIZE) as f32;
        let text_dimensions = measure_text(&text, None, font_size as u16, 1.0);
        let text_x = draw_x - text_dimensions.width / 2.0;
        let text_y = draw_y + text_dimensions.height / 2.0;

        draw_text(&text, text_x, text_y, font_size, SKYBLUE);
    }
}
