use macroquad::prelude::*;

pub fn listem_move() -> (Option<f32>, Option<f32>) {
    let mut moved = false;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;

    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        dy -= 1.0;
        moved = true;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        dy += 1.0;
        moved = true;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        dx -= 1.0;
        moved = true;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        dx += 1.0;
        moved = true;
    }
    if is_key_down(KeyCode::H) {
        dx -= 1.0;
        moved = true;
    }
    if is_key_down(KeyCode::J) {
        dy += 1.0;
        moved = true;
    }
    if is_key_down(KeyCode::K) {
        dy -= 1.0;
        moved = true;
    }
    if is_key_down(KeyCode::L) {
        dx += 1.0;
        moved = true;
    }

    if moved {
        (Some(dx), Some(dy))
    } else {
        (None, None)
    }
}

pub fn listen_quit() -> bool {
    if is_key_down(KeyCode::Q) || is_key_down(KeyCode::Escape) {
        true
    } else {
        false
    }
}
