use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u32,
    pub x: usize,
    pub y: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub players: HashMap<u32, Player>,
}
