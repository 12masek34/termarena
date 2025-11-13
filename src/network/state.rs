use serde::{Deserialize, Serialize};

use crate::game::state::{Direction, GameState, Player};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    InitPlayer(Player),
    GameState(GameState),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Init,
    Quit,
    Move(Direction),
    Shoot,
}
