use std::time::Duration;

pub const PLAYER_RADIUS: f32 = 1.0; // в тайлах
pub const TILE_SIZE: f32 = 10.0;
pub const MAP_WIDTH: usize = 100;
pub const MAP_HEIGHT: usize = 100;
pub const PLAYER_HEALTH: u32 = 3;
pub const WALK_SPEED: f32 = 0.3;
pub const BULLET_SPEED: f32 = 0.2;
pub const BULLET_RANGE: f32 = 15.0;
pub const BULLET_DAMAGE: u32 = 1;
pub const STEP: f32 = 2.0;
pub const HIT_RADIUS: f32 = 0.5;
pub const FIRE_RATE: f32 = 1.0;
pub const MODIFIER_RESPAWN_TIME: Duration = Duration::from_secs(13);

pub const UDP_PORT: usize = 8888;
pub const TCP_PORT: usize = 8887;
