use std::time::Duration;

pub const PLAYER_RADIUS: f32 = 1.0; // в тайлах
pub const TILE_SIZE: f32 = 10.0;
pub const MAP_WIDTH: usize = 1000;
pub const MAP_HEIGHT: usize = 1000;
pub const PLAYER_HEALTH: u32 = 3;
pub const WALK_SPEED: f32 = 50.0;
pub const BULLET_SPEED: f32 = 7.0;
pub const BULLET_RANGE: f32 = 10.0;
pub const BULLET_DAMAGE: u32 = 3;
pub const STEP: f32 = 2.0;
pub const HIT_RADIUS: f32 = 0.5;
pub const FIRE_RATE: f32 = 0.8;
pub const MODIFIER_RESPAWN_TIME: Duration = Duration::from_secs(13);

pub const UDP_PORT: usize = 8888;
pub const TCP_PORT: usize = 8887;
