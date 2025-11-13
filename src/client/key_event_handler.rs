use macroquad::prelude::*;

use crate::game::state::Direction;

pub fn listen_move() -> Option<Direction> {
    let up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) || is_key_down(KeyCode::K);
    let down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) || is_key_down(KeyCode::J);
    let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) || is_key_down(KeyCode::H);
    let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) || is_key_down(KeyCode::L);

    if up {
        Some(Direction::Up)
    } else if down {
        Some(Direction::Down)
    } else if left {
        Some(Direction::Left)
    } else if right {
        Some(Direction::Right)
    } else {
        None
    }
}

pub fn listen_shoot() -> bool {
    is_key_down(KeyCode::Space)
}

pub fn listen_quit() -> bool {
    is_key_down(KeyCode::Q) || is_key_down(KeyCode::Escape)
}
