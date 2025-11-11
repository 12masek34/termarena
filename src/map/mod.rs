use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
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
}
