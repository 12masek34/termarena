use crate::{config::TILE_SIZE, network::state::MapChunk};
use ::rand::Rng;
use ::rand::rngs::ThreadRng;
use ::rand::thread_rng;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Tile {
    Empty,
    Wall,
}

#[derive(Debug)]
pub struct TextureChunk {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub texture: Texture2D,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,

    #[serde(skip)]
    #[serde(default)]
    pub texture_chunks: Arc<Mutex<Option<Vec<TextureChunk>>>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = vec![vec![Tile::Empty; width]; height];
        let mut rng = thread_rng();

        let area = (width * height) as f32;

        let density = 0.05;
        let num_shapes = ((area * density).sqrt() as usize).clamp(5, 100);

        let avg_size = ((width.min(height)) as f32 * 0.5) as usize;
        let max_radius = (width.min(height) / 8).max(3);

        for _ in 0..num_shapes {
            let cx = rng.gen_range(1..width - 1);
            let cy = rng.gen_range(1..height - 1);

            let target_size = rng.gen_range(avg_size / 2..avg_size * 2);

            Self::grow_wall_blob(&mut tiles, cx, cy, target_size, max_radius, &mut rng);
        }

        Self {
            width,
            height,
            tiles,
            texture_chunks: Arc::new(Mutex::new(None)),
        }
    }

    pub fn chunk_map(&self) -> Vec<MapChunk> {
        const CHUNK_SIZE: usize = 1024;

        let raw = bincode::serialize(self).unwrap();
        let total_chunks = (raw.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;

        let mut chunks = Vec::with_capacity(total_chunks);

        for i in 0..total_chunks {
            let start = i * CHUNK_SIZE;
            let end = (start + CHUNK_SIZE).min(raw.len());

            chunks.push(MapChunk {
                chunk_index: i as u32,
                total_chunks: total_chunks as u32,
                bytes: raw[start..end].to_vec(),
            });
        }

        chunks
    }

    pub fn generate_spawn_position(&self, radius: f32) -> (f32, f32) {
        let mut rng = thread_rng();
        loop {
            let x = rng.gen_range(0..self.width) as f32;
            let y = rng.gen_range(0..self.height) as f32;

            let mut can_spawn = true;

            let min_x = (x - radius).floor().max(0.0) as usize;
            let max_x = (x + radius).ceil().min(self.width as f32 - 1.0) as usize;
            let min_y = (y - radius).floor().max(0.0) as usize;
            let max_y = (y + radius).ceil().min(self.height as f32 - 1.0) as usize;

            for ty in min_y..=max_y {
                for tx in min_x..=max_x {
                    if self.tiles[ty][tx] != Tile::Empty {
                        can_spawn = false;
                        break;
                    }
                }
                if !can_spawn {
                    break;
                }
            }

            if can_spawn {
                return (x, y);
            }
        }
    }

    fn grow_wall_blob(
        tiles: &mut Vec<Vec<Tile>>,
        cx: usize,
        cy: usize,
        target_size: usize,
        max_radius: usize,
        rng: &mut ThreadRng,
    ) {
        let height = tiles.len();
        let width = tiles[0].len();
        let mut queue = VecDeque::new();
        queue.push_back((cx, cy));
        let mut count = 0;

        while let Some((x, y)) = queue.pop_front() {
            if count >= target_size {
                break;
            }

            if x >= width || y >= height {
                continue;
            }

            if tiles[y][x] == Tile::Wall {
                continue;
            }

            tiles[y][x] = Tile::Wall;
            count += 1;

            let dirs = [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
            ];

            for (dx, dy) in dirs {
                if rng.gen_bool(0.6) {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;

                    if nx >= 0 && ny >= 0 && nx < width as isize && ny < height as isize {
                        let dist = ((nx - cx as isize).pow(2) + (ny - cy as isize).pow(2)) as f32;
                        if dist.sqrt() <= max_radius as f32 {
                            queue.push_back((nx as usize, ny as usize));
                        }
                    }
                }
            }
        }

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                if tiles[y][x] == Tile::Wall {
                    let neighbors = [
                        &tiles[y - 1][x],
                        &tiles[y + 1][x],
                        &tiles[y][x - 1],
                        &tiles[y][x + 1],
                    ];

                    let wall_neighbors = neighbors.iter().filter(|&&t| *t == Tile::Wall).count();

                    if wall_neighbors < 2 {
                        tiles[y][x] = Tile::Empty;
                    }
                }
            }
        }
    }

    pub fn is_wall(&self, x: f32, y: f32) -> bool {
        let size = 1.0;
        let half = size / 2.0;

        let corners = [
            (x - half, y - half),
            (x + half, y - half),
            (x - half, y + half),
            (x + half, y + half),
        ];

        for &(cx, cy) in &corners {
            if cx < 0.0 || cx >= self.width as f32 || cy < 0.0 || cy >= self.height as f32 {
                return true;
            }

            let tx = if cx > x {
                (cx.ceil() - 1.0) as usize
            } else {
                cx.floor() as usize
            };
            let ty = if cy > y {
                (cy.ceil() - 1.0) as usize
            } else {
                cy.floor() as usize
            };

            if self.tiles[ty][tx] == Tile::Wall {
                return true;
            }
        }

        false
    }

    pub fn init_texture(&self) -> bool {
        const CHUNK_SIZE: usize = 256;
        let mut chunks_guard = self.texture_chunks.lock().unwrap();
        if chunks_guard.is_none() {
            let chunks_x = (self.width + CHUNK_SIZE - 1) / CHUNK_SIZE;
            let chunks_y = (self.height + CHUNK_SIZE - 1) / CHUNK_SIZE;
            let mut result = Vec::new();

            for cy in 0..chunks_y {
                for cx in 0..chunks_x {
                    let start_x = cx * CHUNK_SIZE;
                    let start_y = cy * CHUNK_SIZE;
                    let w = (self.width - start_x).min(CHUNK_SIZE);
                    let h = (self.height - start_y).min(CHUNK_SIZE);

                    let mut image = Image::gen_image_color(w as u16, h as u16, LIGHTGRAY);
                    for y in 0..h {
                        for x in 0..w {
                            let color = match self.tiles[start_y + y][start_x + x] {
                                Tile::Empty => LIGHTGRAY,
                                Tile::Wall => DARKBROWN,
                            };
                            image.set_pixel(x as u32, y as u32, color);
                        }
                    }

                    let texture = Texture2D::from_image(&image);
                    texture.set_filter(FilterMode::Nearest);

                    result.push(TextureChunk {
                        x: start_x,
                        y: start_y,
                        width: w,
                        height: h,
                        texture,
                    });
                }
            }

            *chunks_guard = Some(result);
        }
        return true;
    }

    pub fn render(&self, player_pos: (f32, f32)) {
        self.render_border();
        self.render_texture(player_pos);
    }

    pub fn render_border(&self) {
        draw_rectangle(
            0.0,
            0.0,
            self.width as f32 * TILE_SIZE,
            self.height as f32 * TILE_SIZE,
            BLACK,
        );

        draw_rectangle_lines(
            0.0,
            0.0,
            self.width as f32 * TILE_SIZE,
            self.height as f32 * TILE_SIZE,
            2.0,
            BLACK,
        );
    }

    pub fn render_texture(&self, player_pos: (f32, f32)) {
        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;
        let tiles_in_x = (screen_width() / TILE_SIZE).ceil() as usize;
        let tiles_in_y = (screen_height() / TILE_SIZE).ceil() as usize;
        let start_x = (player_pos.0 as isize - (tiles_in_x / 2) as isize).max(0) as usize;
        let start_y = (player_pos.1 as isize - (tiles_in_y / 2) as isize).max(0) as usize;
        let end_x = (start_x + tiles_in_x).min(self.width);
        let end_y = (start_y + tiles_in_y).min(self.height);
        let offset_x = screen_center_x - (player_pos.0 - start_x as f32) * TILE_SIZE;
        let offset_y = screen_center_y - (player_pos.1 - start_y as f32) * TILE_SIZE;

        let chunks_guard = self.texture_chunks.lock().unwrap();
        if let Some(chunks) = &*chunks_guard {
            for chunk in chunks {
                if chunk.x + chunk.width < start_x
                    || chunk.x > end_x
                    || chunk.y + chunk.height < start_y
                    || chunk.y > end_y
                {
                    continue;
                }

                let src_x = if start_x > chunk.x {
                    (start_x - chunk.x) as f32
                } else {
                    0.0
                };
                let src_y = if start_y > chunk.y {
                    (start_y - chunk.y) as f32
                } else {
                    0.0
                };
                let src_w =
                    ((end_x.min(chunk.x + chunk.width) - (start_x.max(chunk.x))) as f32).max(0.0);
                let src_h =
                    ((end_y.min(chunk.y + chunk.height) - (start_y.max(chunk.y))) as f32).max(0.0);
                let dest_x = offset_x + ((chunk.x.max(start_x) - start_x) as f32 * TILE_SIZE);
                let dest_y = offset_y + ((chunk.y.max(start_y) - start_y) as f32 * TILE_SIZE);

                draw_texture_ex(
                    &chunk.texture,
                    dest_x,
                    dest_y,
                    LIGHTGRAY,
                    DrawTextureParams {
                        source: Some(Rect {
                            x: src_x,
                            y: src_y,
                            w: src_w,
                            h: src_h,
                        }),
                        dest_size: Some(vec2(src_w * TILE_SIZE, src_h * TILE_SIZE)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
