use macroquad::prelude::*;
use std::env;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};
use termarena::client::key_event_handler::{listen_move, listen_quit, listen_shoot};
use termarena::client::state::ClientState;
use termarena::config;
use termarena::network::state::ServerMessage;
use termarena::network::{get_map_from_tcp, recv_message};
use termarena::network::{send_message, state::ClientMessage};

#[macroquad::main("Client")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let server_addr_str = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("127.0.0.1:{}", config::UDP_PORT));
    let server_addr_str = String::from("10.2.106.191:8888");
    let server_addr: SocketAddr = server_addr_str.parse().unwrap();
    let (tx, rx): (Sender<ClientMessage>, Receiver<ClientMessage>) = mpsc::channel();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
    let client_state = Arc::new(Mutex::new(ClientState::new()));

    let client_state_clone_map = Arc::clone(&client_state);
    thread::spawn(move || {
        let map = get_map_from_tcp();
        if let Some(map) = map {
            client_state_clone_map.lock().unwrap().set_map(map);
        }
    });

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

        let direction = listen_move();
        if let Some(direction) = direction {
            let _ = tx.send(ClientMessage::Move(direction));
        }

        if listen_shoot() {
            let _ = tx.send(ClientMessage::Shoot);
        }

        if listen_quit() {
            let _ = tx.send(ClientMessage::Quit);
            break;
        }

        let (gs_arc, map_arc, current_id, player_pos) = {
            let locked_client = client_state.lock().unwrap();
            let player = match locked_client.get_current_player() {
                Some(p) => p.clone(),
                None => {
                    next_frame().await;
                    continue;
                }
            };
            let gs_arc = match &locked_client.game_state {
                Some(gs) => Arc::clone(gs),
                None => {
                    next_frame().await;
                    continue;
                }
            };
            let map_arc = match &locked_client.map {
                Some(map) => Arc::clone(map),
                None => {
                    next_frame().await;
                    continue;
                }
            };
            (gs_arc, map_arc, locked_client.id, (player.x, player.y))
        };

        map_arc.lock().unwrap().render(player_pos);
        gs_arc.render(current_id, player_pos);

        next_frame().await;
    }
}
