use macroquad::prelude::*;
use std::env;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};
use termarena::client::key_event_handler::listem_move;
use termarena::client::state::ClientState;
use termarena::map::Tile;
use termarena::network::recv_message;
use termarena::network::state::ServerMessage;
use termarena::network::{send_message, state::ClientMessage};

#[macroquad::main("Client")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let server_addr_str = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| String::from("127.0.0.1:8888"));
    let server_addr: SocketAddr = server_addr_str.parse().unwrap();
    let (tx, rx): (Sender<ClientMessage>, Receiver<ClientMessage>) = mpsc::channel();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
    let client_state = Arc::new(Mutex::new(ClientState::new()));

    socket
        .set_nonblocking(true)
        .expect("Failed to set nonblocking");
    send_message(&socket, &ClientMessage::Init, server_addr);

    let socket_clone_recv = socket.try_clone().unwrap();
    socket_clone_recv.set_nonblocking(false).unwrap();
    let client_state_clone = Arc::clone(&client_state);

    thread::spawn(move || {
        loop {
            if let Some((msg, _addr)) = recv_message::<ServerMessage>(&socket_clone_recv) {
                match msg {
                    ServerMessage::Map(map) => {
                        client_state_clone.lock().unwrap().set_map(map);
                        println!("Init Map");
                    }
                    ServerMessage::InitPlayer(player) => {
                        client_state_clone.lock().unwrap().init_player(player);
                        println!("Init player");
                    }
                    ServerMessage::GameState(state) => {
                        client_state_clone.lock().unwrap().update_state(state);
                    }
                }
            }
        }
    });

    let socket_clone_send = socket.try_clone().unwrap();
    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            let _ = send_message(&socket_clone_send, &msg, server_addr);
        }
    });

    loop {
        clear_background(BLACK);
        let (gs, map, current_id, player_pos) = {
            let locked_client = client_state.lock().unwrap();
            let player = match locked_client.get_current_player() {
                Some(p) => p.clone(),
                None => continue,
            };
            let gs = match &locked_client.game_state {
                Some(gs) => gs.clone(),
                None => continue,
            };
            let map = match &locked_client.map {
                Some(map) => map.clone(),
                None => continue,
            };
            (gs, map, locked_client.id, (player.x, player.y))
        };

        let (dx, dy) = listem_move();
        if let (Some(dx), Some(dy)) = (dx, dy) {
            let _ = tx.send(ClientMessage::Move(dx, dy));
        }

        let tile_size = 10.0;
        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;

        let tiles_in_x = (screen_width() / tile_size).ceil() as usize;
        let tiles_in_y = (screen_height() / tile_size).ceil() as usize;

        let start_x = (player_pos.0 as isize - (tiles_in_x / 2) as isize).max(0) as usize;
        let start_y = (player_pos.1 as isize - (tiles_in_y / 2) as isize).max(0) as usize;

        let end_x = (start_x + tiles_in_x).min(map.width);
        let end_y = (start_y + tiles_in_y).min(map.height);

        let offset_x = screen_center_x - player_pos.0 * tile_size;
        let offset_y = screen_center_y - player_pos.1 * tile_size;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let draw_x = x as f32 * tile_size + offset_x;
                let draw_y = y as f32 * tile_size + offset_y;
                match map.tiles[y][x] {
                    Tile::Empty => draw_rectangle(draw_x, draw_y, tile_size, tile_size, BLACK),
                    Tile::Wall => draw_rectangle(draw_x, draw_y, tile_size, tile_size, WHITE),
                }
            }
        }

        for player in gs.players.values() {
            let draw_x = player.x * tile_size + offset_x;
            let draw_y = player.y * tile_size + offset_y;
            let color = if Some(player.id) == current_id {
                BLUE
            } else {
                RED
            };
            draw_circle(draw_x, draw_y, tile_size, color);
        }

        next_frame().await;
    }
}
