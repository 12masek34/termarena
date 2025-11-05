use rand::Rng;
use serde::{Deserialize, Serialize};

pub const EMPTY: char = ' ';
pub const WALL: char = '#';

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<char>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = vec![vec![EMPTY; width]; height];
        let mut rng = rand::thread_rng();

        for x in 0..width {
            tiles[0][x] = WALL;
            tiles[height - 1][x] = WALL;
        }
        for y in 0..height {
            tiles[y][0] = WALL;
            tiles[y][width - 1] = WALL;
        }
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                if rng.gen_range(0.0..1.0) < 0.1 {
                    tiles[y][x] = WALL;
                }
            }
        }
        Map {
            width,
            height,
            tiles,
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.tiles[y][x] == EMPTY
    }
}
