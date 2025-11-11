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
            if let Some((msg, addr)) = recv_message::<ServerMessage>(&socket_clone_recv) {
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
                        // println!("{:?}", client_state_clone);
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

        let current_player_opt = {
            let locked_client = client_state.lock().unwrap();
            locked_client.get_current_player()
        };

        let player_pos = if let Some(player) = current_player_opt {
            (player.x, player.y)
        } else {
            continue;
        };

        let (gs, current_id) = {
            let locked_client = client_state.lock().unwrap();
            let gs = match &locked_client.game_state {
                Some(gs) => gs.clone(),
                None => continue,
            };
            (gs, locked_client.id)
        };

        let (dx, dy) = listem_move();

        if let (Some(dx), Some(dy)) = (dx, dy) {
            let _ = tx.send(ClientMessage::Move(dx, dy));
        }

        let tile_size = 10.0;
        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;
        let offset_x = screen_center_x - player_pos.0 * tile_size;
        let offset_y = screen_center_y - player_pos.1 * tile_size;
        let locked_client = client_state.lock().unwrap();
        let map = match &locked_client.map {
            Some(map) => map,
            None => continue,
        };

        for (y, row) in map.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let draw_x = x as f32 * tile_size + offset_x;
                let draw_y = y as f32 * tile_size + offset_y;
                match tile {
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
