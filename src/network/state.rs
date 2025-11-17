use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    game::{player::Player, state::Direction, state::GameState},
    map::Map,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    InitPlayer(Player),
    Map(MapChunk),
    GameState(GameState),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Init,
    Map(HashSet<u32>),
    Quit,
    Move(Direction),
    Shoot,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MapChunk {
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct MapDownloader {
    pub total_chunks: u32,
    pub received: HashMap<u32, Vec<u8>>,
}

impl MapDownloader {
    pub fn new() -> Self {
        Self {
            total_chunks: 0,
            received: HashMap::new(),
        }
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.received.len(), self.total_chunks as usize)
    }

    pub fn get_exist_chunk_id(&self) -> HashSet<u32> {
        self.received.keys().cloned().collect()
    }

    pub fn load_chunk(&mut self, chunk: MapChunk) -> bool {
        self.total_chunks = chunk.total_chunks;
        self.received.insert(chunk.chunk_index, chunk.bytes);

        if let Some(_map) = self.try_build_map() {
            true
        } else {
            false
        }
    }

    pub fn try_build_map(&self) -> Option<Map> {
        if self.received.len() != self.total_chunks as usize {
            return None;
        }

        let mut full = Vec::new();
        for i in 0..self.total_chunks {
            let part = self.received.get(&i).unwrap();
            full.extend_from_slice(part);
        }

        match bincode::deserialize::<Map>(&full) {
            Ok(map) => Some(map),
            Err(_) => None,
        }
    }
}
