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

    #[serde(skip)]
    pub texture: Option<Texture2D>,
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
            texture: None,
        }
    }

    pub fn render(&mut self, player_pos: (f32, f32)) {
        let tile_size = 10.0;
        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;

        let tiles_in_x = (screen_width() / tile_size).ceil() as usize;
        let tiles_in_y = (screen_height() / tile_size).ceil() as usize;

        let start_x = (player_pos.0 as isize - (tiles_in_x / 2) as isize).max(0) as usize;
        let start_y = (player_pos.1 as isize - (tiles_in_y / 2) as isize).max(0) as usize;
        let end_x = (start_x + tiles_in_x).min(self.width);
        let end_y = (start_y + tiles_in_y).min(self.height);

        let offset_x = screen_center_x - (player_pos.0 - start_x as f32) * tile_size;
        let offset_y = screen_center_y - (player_pos.1 - start_y as f32) * tile_size;

        if self.texture.is_none() {
            let mut image = Image::gen_image_color(self.width as u16, self.height as u16, BLACK);

            for y in 0..self.height {
                for x in 0..self.width {
                    let color = match self.tiles[y][x] {
                        Tile::Empty => BLACK,
                        Tile::Wall => WHITE,
                    };
                    image.set_pixel(x as u32, y as u32, color);
                }
            }

            let texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Nearest);
            self.texture = Some(texture);
        }

        if let Some(texture) = &self.texture {
            draw_texture_ex(
                texture,
                offset_x,
                offset_y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect {
                        x: start_x as f32,
                        y: start_y as f32,
                        w: (end_x - start_x) as f32,
                        h: (end_y - start_y) as f32,
                    }),
                    dest_size: Some(vec2(
                        (end_x - start_x) as f32 * tile_size,
                        (end_y - start_y) as f32 * tile_size,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}
