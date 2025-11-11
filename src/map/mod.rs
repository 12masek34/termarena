use ::rand::distributions::{Distribution, Uniform};
use ::rand::thread_rng;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Tile {
    Empty,
    Wall,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = thread_rng();
        let uniform = Uniform::from(0.0f32..1.0f32);

        let mut tiles = vec![vec![Tile::Empty; width]; height];

        for y in 0..height {
            for x in 0..width {
                let roll = uniform.sample(&mut rng);
                tiles[y][x] = if roll < 0.05 { Tile::Wall } else { Tile::Empty };
            }
        }

        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn render(&self, player_pos: (f32, f32)) {
        let tile_size = 10.0;
        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;

        let tiles_in_x = (screen_width() / tile_size).ceil() as usize;
        let tiles_in_y = (screen_height() / tile_size).ceil() as usize;

        let start_x = (player_pos.0 as isize - (tiles_in_x / 2) as isize).max(0) as usize;
        let start_y = (player_pos.1 as isize - (tiles_in_y / 2) as isize).max(0) as usize;

        let end_x = (start_x + tiles_in_x).min(self.width);
        let end_y = (start_y + tiles_in_y).min(self.height);

        let offset_x = screen_center_x - player_pos.0 * tile_size;
        let offset_y = screen_center_y - player_pos.1 * tile_size;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let draw_x = x as f32 * tile_size + offset_x;
                let draw_y = y as f32 * tile_size + offset_y;
                match self.tiles[y][x] {
                    Tile::Empty => draw_rectangle(draw_x, draw_y, tile_size, tile_size, BLACK),
                    Tile::Wall => draw_rectangle(draw_x, draw_y, tile_size, tile_size, WHITE),
                }
            }
        }
    }
}
