use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use crate::config;
use crate::map::Map;
use crate::map::Tile;
use ::rand::Rng;
use ::rand::thread_rng;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub direction: Direction,
    #[serde(skip, default = "Player::default_last_shot")]
    pub last_shot: Instant,
    pub fire_rate: u32,
    pub bullet_speed: f32,
    pub bullet_range: f32,
    pub bullet_damage: u32,
    pub health: u32,
    pub hit_radius: f32,
}

impl Player {
    fn default_last_shot() -> Instant {
        Instant::now() - Duration::from_secs(5)
    }

    pub fn hit_by(&mut self, bullet: &Bullet) -> bool {
        let dx = bullet.x - self.x;
        let dy = bullet.y - self.y;
        if (dx * dx + dy * dy).sqrt() < bullet.hit_radius {
            self.health = self.health.saturating_sub(bullet.damage);
            true
        } else {
            false
        }
    }
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
    pub players: HashMap<u32, Player>,
    pub bullets: HashMap<u32, Bullet>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            bullets: HashMap::new(),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.players.keys().max().map(|id| id + 1).unwrap_or(1)
    }

    pub fn remove(&mut self, id: u32) {
        self.players.remove(&id);
    }

    pub fn get_snapshot(&self) -> Self {
        self.clone()
    }

    pub fn create_player(&mut self, map: &Map) -> Player {
        let id = self.next_id();
        let mut rng = thread_rng();
        let (x, y) = loop {
            let x = rng.gen_range(0..map.width);
            let y = rng.gen_range(0..map.height);

            if map.tiles[y][x] == Tile::Empty {
                break (x as f32, y as f32);
            }
        };
        let player = Player {
            id,
            x,
            y,
            direction: Direction::Up,
            last_shot: Instant::now() - Duration::from_secs(5),
            fire_rate: 1000,
            bullet_speed: 1.0,
            bullet_range: 15.0,
            bullet_damage: 1,
            health: 3,
            hit_radius: 0.5,
        };
        self.players.insert(id, player.clone());

        player
    }

    pub fn move_player(&mut self, player_id: Option<u32>, x: f32, y: f32, map: &Map) {
        if let Some(id) = player_id {
            if let Some(player) = self.players.get_mut(&id) {
                if x > 0.0 {
                    player.direction = Direction::Right;
                } else if x < 0.0 {
                    player.direction = Direction::Left;
                } else if y > 0.0 {
                    player.direction = Direction::Down;
                } else if y < 0.0 {
                    player.direction = Direction::Up;
                }
                let new_x = player.x + x * config::PLAYER_SPEED;
                let new_y = player.y + y * config::PLAYER_SPEED;

                if !map.is_wall(new_x, new_y) {
                    player.x = new_x;
                    player.y = new_y;
                }
            }
        }
    }

    pub fn shoot(&mut self, player_id: Option<u32>) {
        if let Some(id) = player_id {
            let next_bullet_id = self.next_bullet_id();
            if let Some(player) = self.players.get_mut(&id) {
                let fire_rate = Duration::from_millis(player.fire_rate as u64);

                if player.last_shot.elapsed() < fire_rate {
                    return;
                }

                player.last_shot = Instant::now();

                let (dx, dy) = match player.direction {
                    Direction::Up => (0.0, -1.0),
                    Direction::Down => (0.0, 1.0),
                    Direction::Left => (-1.0, 0.0),
                    Direction::Right => (1.0, 0.0),
                };

                let bullet = Bullet {
                    id: next_bullet_id,
                    owner_id: id,
                    x: player.x,
                    y: player.y,
                    dx,
                    dy,
                    speed: player.bullet_speed,
                    range: player.bullet_range,
                    traveled: 0.0,
                    damage: player.bullet_damage,
                    hit_radius: player.hit_radius,
                };

                self.bullets.insert(bullet.id, bullet);
            }
        }
    }

    pub fn next_bullet_id(&self) -> u32 {
        self.bullets.keys().max().map(|id| id + 1).unwrap_or(1)
    }

    pub fn update_bullets(&mut self, map: &Map) {
        let mut to_remove = Vec::new();

        for bullet in self.bullets.values_mut() {
            bullet.x += bullet.dx * bullet.speed;
            bullet.y += bullet.dy * bullet.speed;
            bullet.traveled += bullet.speed;

            if map.is_wall(bullet.x, bullet.y) || bullet.traveled >= bullet.range {
                to_remove.push(bullet.id);
                continue;
            }

            for (player_id, player) in self.players.iter_mut() {
                if bullet.owner_id != *player_id && player.hit_by(bullet) {
                    to_remove.push(bullet.id);
                    break;
                }
            }
        }

        for id in to_remove {
            self.bullets.remove(&id);
        }
    }

    pub fn render(&self, current_id: Option<u32>, player_pos: (f32, f32)) {
        let offset_x = screen_width() / 2.0 - player_pos.0 * config::TILE_SIZE;
        let offset_y = screen_height() / 2.0 - player_pos.1 * config::TILE_SIZE;

        for player in self.players.values() {
            let draw_x = player.x * config::TILE_SIZE + offset_x;
            let draw_y = player.y * config::TILE_SIZE + offset_y;
            let color = if Some(player.id) == current_id {
                BLUE
            } else {
                RED
            };
            draw_circle(draw_x, draw_y, config::TILE_SIZE, color);
        }

        for bullet in self.bullets.values() {
            let draw_x = bullet.x * config::TILE_SIZE + offset_x;
            let draw_y = bullet.y * config::TILE_SIZE + offset_y;
            draw_circle(draw_x, draw_y, config::TILE_SIZE / 4.0, DARKPURPLE);
        }
    }
}
