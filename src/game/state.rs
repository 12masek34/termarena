use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use crate::config;
use crate::map::Map;

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
    pub kills: u32,
    pub deths: u32,
    pub radius: f32,
    pub direction: Direction,
    pub fire_rate: u32,
    pub bullet_speed: f32,
    pub bullet_range: f32,
    pub bullet_damage: u32,
    pub health: u32,
    pub max_health: u32,
    pub hit_radius: f32,
    pub is_moving: bool,
    pub move_target: Option<(f32, f32)>,
    pub walk_speed: f32,

    #[serde(skip, default = "Player::default_last_shot")]
    pub last_shot: Instant,
}

impl Player {
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

impl Bullet {
    fn render(&self, offset_x: f32, offset_y: f32) {
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

    pub fn remove(&mut self, player_id: Option<&u32>) {
        if let Some(id) = player_id {
            self.players.remove(&id);
        }
    }

    pub fn get_snapshot(&self) -> Self {
        self.clone()
    }

    pub fn create_player(&mut self, map: &Map) -> Player {
        let id = self.next_id();
        let (x, y) = map.generate_spawn_position(config::PLAYER_RADIUS);
        let player = Player {
            id,
            x,
            y,
            kills: 0,
            deths: 0,
            radius: config::PLAYER_RADIUS,
            direction: Direction::Up,
            last_shot: Instant::now() - Duration::from_secs(5),
            fire_rate: 1000,
            bullet_speed: config::BULLET_SPEED,
            bullet_range: config::BULLET_RANGE,
            bullet_damage: 1,
            health: config::PLAYER_HEALTH,
            max_health: config::PLAYER_HEALTH,
            hit_radius: config::HIT_RADIUS,
            is_moving: false,
            move_target: None,
            walk_speed: config::WALK_SPEED,
        };
        self.players.insert(id, player.clone());

        player
    }

    pub fn move_player(&mut self, player_id: Option<&u32>, dir: Direction, map: &Map) {
        if let Some(id) = player_id {
            if let Some(player) = self.players.get_mut(id) {
                if player.is_moving {
                    return;
                }

                player.direction = dir;

                let (dx, dy) = match player.direction {
                    Direction::Up => (0.0, -config::STEP),
                    Direction::Down => (0.0, config::STEP),
                    Direction::Left => (-config::STEP, 0.0),
                    Direction::Right => (config::STEP, 0.0),
                };

                let new_x = player.x + dx * 0.5;
                let new_y = player.y + dy * 0.5;

                if !map.is_wall(new_x, new_y) {
                    player.move_target = Some((new_x, new_y));
                    player.is_moving = true;
                }
            }
        }
    }

    pub fn shoot(&mut self, player_id: Option<&u32>) {
        if let Some(id) = player_id {
            let next_bullet_id = self.next_bullet_id();
            if let Some(player) = self.players.get_mut(&id) {
                let fire_rate = Duration::from_millis(player.fire_rate as u64);

                if player.last_shot.elapsed() < fire_rate {
                    return;
                }

                player.last_shot = Instant::now();

                let (dx, dy) = match player.direction {
                    Direction::Up => (0.0, -config::STEP),
                    Direction::Down => (0.0, config::STEP),
                    Direction::Left => (-config::STEP, 0.0),
                    Direction::Right => (config::STEP, 0.0),
                };

                let bullet = Bullet {
                    id: next_bullet_id,
                    owner_id: *id,
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
        let mut to_respawn = Vec::new();

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

                    if player.health == 0 {
                        to_respawn.push(*player_id);
                        let bullet_owner = self.players.get_mut(&bullet.owner_id);

                        if let Some(owner) = bullet_owner {
                            owner.kills += 1;
                        }
                    }

                    break;
                }
            }
        }

        for id in to_remove {
            self.bullets.remove(&id);
        }

        for player_id in to_respawn {
            self.respawn(player_id, map);
        }
    }

    pub fn update_players(&mut self, map: &Map) {
        for player in self.players.values_mut() {
            if player.is_moving {
                let (tx, ty) = player.move_target.unwrap();
                let dx = tx - player.x;
                let dy = ty - player.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= player.walk_speed {
                    player.x = tx;
                    player.y = ty;
                    player.is_moving = false;
                    player.move_target = None;
                } else {
                    player.x += player.walk_speed * dx / dist;
                    player.y += player.walk_speed * dy / dist;
                }

                if map.is_wall(player.x, player.y) {
                    player.is_moving = false;
                    player.move_target = None;
                }
            }
        }
    }

    pub fn update(&mut self, map: &Map) {
        self.update_bullets(map);
        self.update_players(map);
    }

    pub fn respawn(&mut self, player_id: u32, map: &Map) {
        if let Some(player) = self.players.get_mut(&player_id) {
            let (x, y) = map.generate_spawn_position(player.radius);
            player.x = x;
            player.y = y;
            player.health = config::PLAYER_HEALTH;
            player.direction = Direction::Up;
            player.last_shot = Instant::now() - Duration::from_secs(5);
            player.deths += 1;
        }
    }

    pub fn render(&self, current_id: Option<u32>, player_pos: (f32, f32)) {
        let offset_x = screen_width() / 2.0 - player_pos.0 * config::TILE_SIZE;
        let offset_y = screen_height() / 2.0 - player_pos.1 * config::TILE_SIZE;

        for player in self.players.values() {
            player.render(current_id, offset_x, offset_y);
        }

        for bullet in self.bullets.values() {
            bullet.render(offset_x, offset_y);
        }

        self.render_hud(current_id);
    }

    pub fn render_hud(&self, current_id: Option<u32>) {
        let mut players: Vec<_> = self.players.values().collect();
        players.sort_by_key(|p| p.id);

        let mut y = 10.0;
        draw_text(
            &format!("Players online: {}", players.len()),
            10.0,
            y,
            20.0,
            WHITE,
        );
        y += 25.0;

        for player in players {
            let current_marker = if Some(player.id) == current_id {
                "(You)"
            } else {
                ""
            };
            draw_text(
                &format!(
                    "ID: {} {} | Kills: {} | Deths: {}",
                    player.id, current_marker, player.kills, player.deths
                ),
                10.0,
                y,
                20.0,
                WHITE,
            );
            y += 25.0;
        }
    }
}
