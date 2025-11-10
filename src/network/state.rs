use serde::{Deserialize, Serialize};

use crate::game::state::{GameState, Player};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    InitPlayer(Player),
    GameState(GameState),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Init,
    Quit,
    Move(i32, i32),
}
