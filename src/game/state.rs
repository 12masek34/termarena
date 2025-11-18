use ::rand::Rng;
use ::rand::thread_rng;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use crate::config;
use crate::game::modifier::ModifierKind;
use crate::map::Map;

use super::bullet::Bullet;
use super::modifier::Modifier;
use super::player::Player;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameStateDiff {
    pub players: HashMap<u32, Player>,
    pub removed_players: Vec<u32>,
    pub bullets: HashMap<u32, Bullet>,
    pub removed_bullets: Vec<u32>,
    pub modifieres: HashMap<u32, Modifier>,
    pub removed_modifieres: Vec<u32>,
}

impl GameStateDiff {
    fn new() -> Self {
        Self {
            players: HashMap::new(),
            removed_players: Vec::new(),
            bullets: HashMap::new(),
            removed_bullets: Vec::new(),
            modifieres: HashMap::new(),
            removed_modifieres: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerPrevState {
    players: HashMap<u32, Player>,
    bullets: HashMap<u32, Bullet>,
    modifieres: HashMap<u32, Modifier>,
}

impl PlayerPrevState {
    pub fn new() -> Self {
        PlayerPrevState {
            players: HashMap::new(),
            bullets: HashMap::new(),
            modifieres: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
    pub players: HashMap<u32, Player>,
    pub bullets: HashMap<u32, Bullet>,
    pub modifieres: HashMap<u32, Modifier>,

    #[serde(skip_serializing, skip_deserializing, default = "Instant::now")]
    pub last_spawn_modifieres: Instant,

    #[serde(skip_serializing, skip_deserializing, default)]
    pub prev_states: HashMap<u32, Box<PlayerPrevState>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            bullets: HashMap::new(),
            modifieres: HashMap::new(),
            last_spawn_modifieres: Instant::now(),
            prev_states: HashMap::new(),
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

    pub fn get_snapshot(&mut self, player_id: Option<&u32>) -> Self {
        if let Some(pid) = player_id {
            if let Some(player) = self.players.get(pid) {
                let half_w = 30.0; // TODO: реальные размеры экрана
                let half_h = 20.0;
                let px = player.x;
                let py = player.y;

                let mut player_prev = PlayerPrevState {
                    players: HashMap::new(),
                    bullets: HashMap::new(),
                    modifieres: HashMap::new(),
                };

                for (&id, p) in &self.players {
                    if self.is_in_viewport(px, py, p.x, p.y, half_w, half_h) {
                        player_prev.players.insert(id, p.clone());
                    }
                }
                for (&id, b) in &self.bullets {
                    if self.is_in_viewport(px, py, b.x, b.y, half_w, half_h) {
                        player_prev.bullets.insert(id, b.clone());
                    }
                }
                for (&id, m) in &self.modifieres {
                    if self.is_in_viewport(px, py, m.x, m.y, half_w, half_h) {
                        player_prev.modifieres.insert(id, m.clone());
                    }
                }
                self.prev_states.insert(*pid, Box::new(player_prev));

                return self.clone();
            }
        }

        self.clone()
    }

    pub fn full_snapshot(&mut self) -> GameStateDiff {
        let mut diff = GameStateDiff::new();
        diff.players = self.players.clone();
        diff.bullets = self.bullets.clone();
        diff.modifieres = self.modifieres.clone();
        diff
    }

    pub fn get_snapshot_diff(&mut self, player_id: Option<&u32>) -> GameStateDiff {
        let mut diff = GameStateDiff::new();

        if player_id.is_none() {
            return self.full_snapshot();
        }

        let Some((px, py, half_w, half_h, pid)) = self.resolve_viewport(player_id.unwrap()) else {
            return diff;
        };

        let prev = self.prev_states.get(&pid);

        self.collect_player_changes(&mut diff, pid, px, py, half_w, half_h, prev);
        self.collect_bullet_changes(&mut diff, px, py, half_w, half_h, prev);
        self.collect_modifier_changes(&mut diff, px, py, half_w, half_h, prev);

        let new_prev = self.build_new_prev_state(px, py, half_w, half_h);
        self.prev_states.insert(pid, Box::new(new_prev));

        diff
    }

    pub fn resolve_viewport(&self, pid: &u32) -> Option<(f32, f32, f32, f32, u32)> {
        self.players.get(pid).map(|player| {
            let half_w = 30.0;
            let half_h = 20.0;
            (player.x, player.y, half_w, half_h, *pid)
        })
    }

    pub fn collect_player_changes(
        &self,
        diff: &mut GameStateDiff,
        _pid: u32,
        px: f32,
        py: f32,
        half_w: f32,
        half_h: f32,
        prev: Option<&Box<PlayerPrevState>>,
    ) {
        for (&id, player) in &self.players {
            let changed = prev.map_or(true, |p| p.players.get(&id) != Some(player));
            if changed {
                diff.players.insert(id, player.clone());
            }
        }

        if let Some(prev_state) = prev {
            for (&id, _) in &prev_state.players {
                let removed = !self.players.contains_key(&id)
                    || !self
                        .players
                        .get(&id)
                        .map(|p| self.is_in_viewport(px, py, p.x, p.y, half_w, half_h))
                        .unwrap_or(false);

                if removed {
                    diff.removed_players.push(id);
                }
            }
        }
    }

    pub fn collect_bullet_changes(
        &self,
        diff: &mut GameStateDiff,
        px: f32,
        py: f32,
        half_w: f32,
        half_h: f32,
        prev: Option<&Box<PlayerPrevState>>,
    ) {
        for (&id, bullet) in &self.bullets {
            if self.is_in_viewport(px, py, bullet.x, bullet.y, half_w, half_h) {
                let changed = prev.map_or(true, |p| p.bullets.get(&id) != Some(bullet));
                if changed {
                    diff.bullets.insert(id, bullet.clone());
                }
            }
        }

        if let Some(prev_state) = prev {
            for (&id, _) in &prev_state.bullets {
                let removed = !self.bullets.contains_key(&id)
                    || !self
                        .bullets
                        .get(&id)
                        .map(|b| self.is_in_viewport(px, py, b.x, b.y, half_w, half_h))
                        .unwrap_or(false);

                if removed {
                    diff.removed_bullets.push(id);
                }
            }
        }
    }

    pub fn collect_modifier_changes(
        &self,
        diff: &mut GameStateDiff,
        px: f32,
        py: f32,
        half_w: f32,
        half_h: f32,
        prev: Option<&Box<PlayerPrevState>>,
    ) {
        for (&id, modifier) in &self.modifieres {
            if self.is_in_viewport(px, py, modifier.x, modifier.y, half_w, half_h) {
                let changed = prev.map_or(true, |p| p.modifieres.get(&id) != Some(modifier));
                if changed {
                    diff.modifieres.insert(id, modifier.clone());
                }
            }
        }

        if let Some(prev_state) = prev {
            for (&id, _) in &prev_state.modifieres {
                let removed = prev_state.modifieres.contains_key(&id)
                    && (!self.modifieres.contains_key(&id)
                        || !self.is_in_viewport(
                            px,
                            py,
                            self.modifieres.get(&id).map(|m| m.x).unwrap_or(0.0),
                            self.modifieres.get(&id).map(|m| m.y).unwrap_or(0.0),
                            half_w,
                            half_h,
                        ));

                if removed {
                    diff.removed_modifieres.push(id);
                }
            }
        }
    }

    pub fn build_new_prev_state(
        &self,
        px: f32,
        py: f32,
        half_w: f32,
        half_h: f32,
    ) -> PlayerPrevState {
        let mut new_prev = PlayerPrevState::new();

        for (&id, player) in &self.players {
            if self.is_in_viewport(px, py, player.x, player.y, half_w, half_h) {
                new_prev.players.insert(id, player.clone());
            }
        }
        for (&id, bullet) in &self.bullets {
            if self.is_in_viewport(px, py, bullet.x, bullet.y, half_w, half_h) {
                new_prev.bullets.insert(id, bullet.clone());
            }
        }
        for (&id, modifier) in &self.modifieres {
            if self.is_in_viewport(px, py, modifier.x, modifier.y, half_w, half_h) {
                new_prev.modifieres.insert(id, modifier.clone());
            }
        }

        new_prev
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
                let fire_interval = Duration::from_secs_f32(1.0 / player.fire_rate);

                if player.last_shot.elapsed() < fire_interval {
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

    pub fn update_bullets(&mut self, map: &Map, delta_time: f32) {
        let mut to_remove = Vec::new();
        let mut to_respawn = Vec::new();

        for bullet in self.bullets.values_mut() {
            let step = bullet.speed * delta_time;
            bullet.x += bullet.dx * step;
            bullet.y += bullet.dy * step;
            bullet.traveled += step;

            if map.is_wall(bullet.x, bullet.y) || bullet.traveled >= bullet.range {
                to_remove.push(bullet.id);
                continue;
            }

            for (player_id, player) in self.players.iter_mut() {
                if bullet.owner_id != *player_id && player.hit_by(bullet) {
                    to_remove.push(bullet.id);

                    if player.health == 0 {
                        to_respawn.push(*player_id);
                        if let Some(owner) = self.players.get_mut(&bullet.owner_id) {
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

    pub fn update_players(&mut self, map: &Map, delta_time: f32) {
        let mut picked_modifiers = Vec::new();

        for player in self.players.values_mut() {
            if player.is_moving {
                let (tx, ty) = player.move_target.unwrap();
                let dx = tx - player.x;
                let dy = ty - player.y;
                let dist = (dx * dx + dy * dy).sqrt();

                let step = player.walk_speed * delta_time;
                if dist <= step {
                    player.x = tx;
                    player.y = ty;
                    player.is_moving = false;
                    player.move_target = None;
                } else {
                    let next_x = player.x + step * dx / dist;
                    let next_y = player.y + step * dy / dist;

                    if !map.is_wall(next_x, next_y) {
                        player.x = next_x;
                        player.y = next_y;
                    } else {
                        player.is_moving = false;
                        player.move_target = None;
                    }
                }
            }

            if map.is_wall(player.x, player.y) {
                player.is_moving = false;
                player.move_target = None;
            }

            for (id, modifier) in &self.modifieres {
                let dx = modifier.x - player.x;
                let dy = modifier.y - player.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < 1.0 {
                    picked_modifiers.push(*id);
                    match modifier.kind {
                        ModifierKind::Heal(health) => {
                            player.health += health;
                            player.max_health += health;
                        }
                        ModifierKind::Speed(speed) => {
                            player.walk_speed += speed;
                        }
                        ModifierKind::Damage(damage) => {
                            player.bullet_damage += damage;
                        }
                        ModifierKind::FireRate(fire_rate) => {
                            player.fire_rate += fire_rate;
                        }
                        ModifierKind::BulletRange(bullet_range) => {
                            player.bullet_range += bullet_range;
                        }
                    }
                }
            }
        }
        for id in picked_modifiers {
            self.modifieres.remove(&id);
        }
    }

    pub fn update(&mut self, map: &Map, delta_time: f32) {
        self.update_bullets(map, delta_time);
        self.update_players(map, delta_time);
        self.spawn_modifiers(map);
    }

    pub fn spawn_modifiers(&mut self, map: &Map) {
        if self.last_spawn_modifieres.elapsed() < config::MODIFIER_RESPAWN_TIME {
            return;
        }
        self.last_spawn_modifieres = Instant::now();
        self.modifieres.clear();
        let mut rng = thread_rng();
        let modifiers_count = ((map.width * map.height) as f32 / 500.0).round() as u32;

        for id in 0..modifiers_count {
            let (mut x, mut y);

            loop {
                x = rng.gen_range(0..map.width) as f32 + 0.5;
                y = rng.gen_range(0..map.height) as f32 + 0.5;
                if !map.is_wall(x, y) {
                    break;
                }
            }

            let modifier = Modifier {
                id,
                x,
                y,
                kind: ModifierKind::random(&mut rng),
            };

            self.modifieres.insert(id, modifier);
        }
    }

    pub fn respawn(&mut self, player_id: u32, map: &Map) {
        if let Some(player) = self.players.get_mut(&player_id) {
            let (x, y) = map.generate_spawn_position(player.radius);
            player.x = x;
            player.y = y;
            player.health = player.max_health;
            player.direction = Direction::Up;
            player.last_shot = Instant::now() - Duration::from_secs(5);
            player.deths += 1;
        }
    }

    pub fn render(&self, current_id: Option<u32>, player_pos: (f32, f32)) {
        let offset_x = screen_width() / 2.0 - player_pos.0 * config::TILE_SIZE;
        let offset_y = screen_height() / 2.0 - player_pos.1 * config::TILE_SIZE;

        for player in self.players.values() {
            if Some(player.id) == current_id {
                player.render(current_id, offset_x, offset_y);
                continue;
            }

            let dx = player.x - player_pos.0;
            let dy = player.y - player_pos.1;
            let screen_x = screen_width() / 2.0 + dx * config::TILE_SIZE;
            let screen_y = screen_height() / 2.0 + dy * config::TILE_SIZE;

            if screen_x < 0.0
                || screen_x > screen_width()
                || screen_y < 0.0
                || screen_y > screen_height()
            {
                self.draw_offscreen_arrow(dx, dy);
            } else {
                player.render(current_id, offset_x, offset_y);
            }
        }

        for bullet in self.bullets.values() {
            bullet.render(offset_x, offset_y);
        }

        for modifier in self.modifieres.values() {
            modifier.render(offset_x, offset_y);
        }

        self.render_hud(current_id);
    }

    pub fn draw_offscreen_arrow(&self, dx: f32, dy: f32) {
        let angle = dy.atan2(dx);
        let margin = 20.0;
        let half_w = screen_width() / 2.0 - margin;
        let half_h = screen_height() / 2.0 - margin;

        let mut arrow_x = screen_width() / 2.0 + half_w * angle.cos();
        let mut arrow_y = screen_height() / 2.0 + half_h * angle.sin();

        arrow_x = arrow_x.clamp(margin, screen_width() - margin);
        arrow_y = arrow_y.clamp(margin, screen_height() - margin);

        let arrow_size = 10.0;
        let angle_offset = std::f32::consts::PI / 6.0;

        let tip = Vec2::new(arrow_x, arrow_y);
        let left = Vec2::new(
            arrow_x - arrow_size * (angle - angle_offset).cos(),
            arrow_y - arrow_size * (angle - angle_offset).sin(),
        );
        let right = Vec2::new(
            arrow_x - arrow_size * (angle + angle_offset).cos(),
            arrow_y - arrow_size * (angle + angle_offset).sin(),
        );

        draw_triangle(tip, left, right, RED);
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
                "     "
            };
            draw_text(
                &format!(
                    "ID: {}{} | Kills: {} | Deths: {} | Health {}/{} | speed {} | Damage: {}| fire rate {}",
                    player.id,
                    current_marker,
                    player.kills,
                    player.deths,
                    player.health,
                    player.max_health,
                    player.walk_speed,
                    player.bullet_damage,
                    player.fire_rate,
                ),
                10.0,
                y,
                20.0,
                WHITE,
            );
            y += 25.0;
        }
    }

    pub fn is_in_viewport(
        &self,
        px: f32,
        py: f32,
        ox: f32,
        oy: f32,
        half_w: f32,
        half_h: f32,
    ) -> bool {
        let dx = (ox - px).abs();
        let dy = (oy - py).abs();
        dx <= half_w && dy <= half_h
    }
}
