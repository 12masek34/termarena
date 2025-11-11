use crate::map::Tile;
use macroquad::prelude::*;
use std::sync::Arc;

use crate::{
    game::state::{GameState, Player},
    map::Map,
};

#[derive(Debug)]
pub struct ClientState {
    pub id: Option<u32>,
    pub map: Option<Arc<Map>>,
    pub game_state: Option<Arc<GameState>>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            id: None,
            map: None,
            game_state: None,
        }
    }

    pub fn init_player(&mut self, player: Player) {
        if self.id.is_none() {
            self.id = Some(player.id);
        }
    }

    pub fn update_state(&mut self, state: GameState) {
        self.game_state = Some(Arc::new(state));
    }

    pub fn get_current_player(&self) -> Option<Player> {
        if let Some(gs) = &self.game_state {
            self.id.and_then(|id| gs.players.get(&id).cloned())
        } else {
            None
        }
    }

    pub fn set_map(&mut self, map: Map) {
        if self.map.is_none() {
            self.map = Some(Arc::new(map));
        }
    }

    pub fn render_map(&self, map_arc: &Arc<Map>, player_pos: (f32, f32)) {
        let tile_size = 10.0;
        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;

        let tiles_in_x = (screen_width() / tile_size).ceil() as usize;
        let tiles_in_y = (screen_height() / tile_size).ceil() as usize;

        let start_x = (player_pos.0 as isize - (tiles_in_x / 2) as isize).max(0) as usize;
        let start_y = (player_pos.1 as isize - (tiles_in_y / 2) as isize).max(0) as usize;

        let end_x = (start_x + tiles_in_x).min(map_arc.width);
        let end_y = (start_y + tiles_in_y).min(map_arc.height);

        let offset_x = screen_center_x - player_pos.0 * tile_size;
        let offset_y = screen_center_y - player_pos.1 * tile_size;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let draw_x = x as f32 * tile_size + offset_x;
                let draw_y = y as f32 * tile_size + offset_y;
                match map_arc.tiles[y][x] {
                    Tile::Empty => draw_rectangle(draw_x, draw_y, tile_size, tile_size, BLACK),
                    Tile::Wall => draw_rectangle(draw_x, draw_y, tile_size, tile_size, WHITE),
                }
            }
        }
    }

    pub fn render_game_state(
        &self,
        game_state_arc: &Arc<GameState>,
        current_id: Option<u32>,
        player_pos: (f32, f32),
    ) {
        let tile_size = 10.0;
        let offset_x = screen_width() / 2.0 - player_pos.0 * tile_size;
        let offset_y = screen_height() / 2.0 - player_pos.1 * tile_size;

        for player in game_state_arc.players.values() {
            let draw_x = player.x * tile_size + offset_x;
            let draw_y = player.y * tile_size + offset_y;
            let color = if Some(player.id) == current_id {
                BLUE
            } else {
                RED
            };
            draw_circle(draw_x, draw_y, tile_size, color);
        }
    }
}
