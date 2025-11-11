use macroquad::prelude::*;
use std::env;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};
use termarena::client::state::ClientState;
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

    thread::spawn(move || {
        loop {
            if let Some((msg, addr)) = recv_message::<ServerMessage>(&socket_clone_recv) {
                match msg {
                    ServerMessage::Map => {
                        println!("MAP");
                    }
                    ServerMessage::InitPlayer(player) => {
                        client_state.lock().unwrap().init_player(player);
                    }
                    ServerMessage::GameState(state) => {
                        client_state.lock().unwrap().update_state(state);
                        println!("{:?}", client_state);
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

    let mut player_pos = (100.0f32, 100.0f32);
    loop {
        clear_background(BLACK);
        let mut moved = false;
        let mut dx: f32 = 0.0;
        let mut dy: f32 = 0.0;

        if is_key_down(KeyCode::W) {
            dy -= 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::S) {
            dy += 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::A) {
            dx -= 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::D) {
            dx += 1.0;
            moved = true;
        }

        if moved {
            player_pos.0 += dx as f32 * 5.0;
            player_pos.1 += dy as f32 * 5.0;
            let _ = tx.send(ClientMessage::Move(dx, dy));
        }
        draw_circle(player_pos.0, player_pos.1, 20.0, BLUE);
        next_frame().await;
    }
}
