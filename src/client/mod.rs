pub mod ui;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use crate::map::Map;
use crate::network::receive_map;
use std::time::Duration;
use tokio::time::sleep;


pub struct ClientState {
    pub map: Map,
    pub player_x: usize,
    pub player_y: usize,
}

impl ClientState {
    pub fn new(map: Map) -> Self {
        let mut spawn_x = 1;
        let mut spawn_y = 1;

        for y in 1..map.height {
            for  x in 1..map.width {
                if map.tiles[y][x] == crate::map::EMPTY {
                    spawn_x = x;
                    spawn_y = y;
                    break;
                }
            }
        }
        Self {
            map,
            player_x: spawn_x,
            player_y: spawn_y,
        }
    }

}


pub async fn run_client(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = match TcpStream::connect(addr).await {
    Ok(s) => s,
    Err(e) => {
        eprintln!("Ошибка подключения к серверу {}: {}", addr, e);
        return Err(Box::new(e));
    }
};
    println!("Подключено к серверу {}", addr);

    let map = receive_map(&mut stream).await?;
    println!("Карта получена: {}x{}", map.width, map.height);

    let mut state = ClientState::new(map);
    run_game_loop(&mut state).await;

    Ok(())
}

async fn run_game_loop(state: &ClientState) {
    loop {
        print!("\x1B[2J\x1B[H");

        ui::render(&state);
        sleep(Duration::from_millis(500)).await;
    }
}
