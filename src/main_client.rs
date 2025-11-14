use macroquad::prelude::*;
use std::env;
use std::time::Duration;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};
use termarena::client::key_event_handler::{listen_move, listen_quit, listen_shoot};
use termarena::client::state::ClientState;
use termarena::config;
use termarena::network::recv_message;
use termarena::network::state::ServerMessage;
use termarena::network::{send_message, state::ClientMessage, state::MapDownloader};

#[macroquad::main("Client")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let server_addr_str = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("127.0.0.1:{}", config::UDP_PORT));
    // let server_addr_str = String::from("10.2.106.191:8888");
    let server_addr: SocketAddr = server_addr_str.parse().unwrap();
    let (tx, rx): (Sender<ClientMessage>, Receiver<ClientMessage>) = mpsc::channel();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
    let client_state = Arc::new(Mutex::new(ClientState::new()));
    let map = Arc::new(Mutex::new(None));
    let map_clone = Arc::clone(&map);
    let map_downloader = Arc::new(Mutex::new(MapDownloader::new()));
    let map_downloader_recv = Arc::clone(&map_downloader);

    socket
        .set_nonblocking(false)
        .expect("Failed to set nonblocking");
    send_message(&socket, &ClientMessage::Init, server_addr);

    let socket_clone = socket.try_clone().unwrap();
    let map_clone_check = Arc::clone(&map);
    thread::spawn(move || {
        loop {
            let map_ready = {
                let map = map_clone_check.lock().unwrap();
                map.is_some()
            };
            if map_ready {
                break;
            }

            send_message(&socket_clone, &ClientMessage::Map, server_addr);
            thread::sleep(Duration::from_secs(30));
        }
    });

    let socket_clone_recv = socket.try_clone().unwrap();
    socket_clone_recv
        .set_nonblocking(false)
        .expect("Failed to set blocking mode");
    let client_state_clone = Arc::clone(&client_state);
    thread::spawn(move || {
        loop {
            if let Some((msg, _addr)) = recv_message::<ServerMessage>(&socket_clone_recv) {
                let mut clinet_state_clone_lock = client_state_clone.lock().unwrap();
                match msg {
                    ServerMessage::InitPlayer(player) => {
                        clinet_state_clone_lock.init_player(player);
                    }
                    ServerMessage::Map(chunk) => {
                        let mut map_downloader_lock = map_downloader_recv.lock().unwrap();
                        if let Some(new_map) = map_downloader_lock.load_chunk(chunk) {
                            *map_clone.lock().unwrap() = Some(Arc::new(new_map));
                        }
                    }
                    ServerMessage::GameState(state) => {
                        clinet_state_clone_lock.update_state(state);
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

    let mut last_update = std::time::Instant::now();
    let mut loading_frame = 0;

    next_frame().await;

    loop {
        clear_background(BLACK);

        if let Some(direction) = listen_move() {
            let _ = tx.send(ClientMessage::Move(direction));
        }
        if listen_shoot() {
            let _ = tx.send(ClientMessage::Shoot);
        }
        if listen_quit() {
            let _ = tx.send(ClientMessage::Quit);
            break;
        }

        let locked_client = client_state.lock().unwrap();
        let map_ready = map.lock().unwrap().is_some();
        let gs_ready = locked_client.game_state.is_some();
        let player_ready = locked_client.get_current_player().is_some();

        if map_ready && gs_ready && player_ready {
            let player = locked_client.get_current_player().unwrap().clone();
            let gs_arc = Arc::clone(locked_client.game_state.as_ref().unwrap());
            let current_id = locked_client.id;
            drop(locked_client);

            if let Some(map_arc) = map.lock().unwrap().as_ref() {
                map_arc.init_texture();
                map_arc.render((player.x, player.y));
            }

            gs_arc.render(current_id, (player.x, player.y));
        } else {
            if last_update.elapsed() > std::time::Duration::from_millis(300) {
                loading_frame += 1;
                last_update = std::time::Instant::now();
            }
            let loading_text = format!("Loading{}", ".".repeat(loading_frame % 4));
            draw_text(&loading_text, 20.0, 50.0, 30.0, WHITE);

            let (received, total) = {
                let dl = map_downloader.lock().unwrap();
                dl.progress()
            };

            if total > 0 {
                let percent = received as f32 / total as f32 * 100.0;
                let prog_text =
                    format!("Downloading map: {}/{} ({:.1}%)", received, total, percent);
                draw_text(&prog_text, 20.0, 90.0, 25.0, WHITE);

                let bar_width = 300.0;
                let filled = bar_width * (received as f32 / total as f32);

                draw_rectangle(20.0, 110.0, bar_width, 20.0, GRAY);
                draw_rectangle(20.0, 110.0, filled, 20.0, GREEN);
            }
        }

        next_frame().await;
    }
}
