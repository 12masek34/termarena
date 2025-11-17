use macroquad::prelude::*;
use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};
use termarena::client::key_event_handler::{listen_move, listen_quit, listen_shoot};
use termarena::client::state::ClientState;
use termarena::config;
use termarena::map::Map;
use termarena::network::recv_message;
use termarena::network::state::ServerMessage;
use termarena::network::{send_message, state::ClientMessage, state::MapDownloader};
use termarena::ui::loading;

#[macroquad::main("Client")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let server_addr_str = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("127.0.0.1:{}", config::UDP_PORT));
    let server_addr: SocketAddr = server_addr_str.parse().unwrap();
    let (tx, rx): (Sender<ClientMessage>, Receiver<ClientMessage>) = mpsc::channel();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
    let client_state = Arc::new(Mutex::new(ClientState::new()));
    let map: Arc<Mutex<Option<Arc<Map>>>> = Arc::new(Mutex::new(None));
    let map_downloader = Arc::new(Mutex::new(MapDownloader::new()));
    let map_downloader_recv = Arc::clone(&map_downloader);
    let map_downloader_send = Arc::clone(&map_downloader);
    let map_loaded = Arc::new(AtomicBool::new(false));
    let map_loaded_clone = Arc::clone(&map_loaded);
    let mut texture_inited = false;

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

            let map_chunk_ids = { map_downloader_send.lock().unwrap().get_exist_chunk_id() };

            send_message(
                &socket_clone,
                &ClientMessage::Map(map_chunk_ids),
                server_addr,
            );
            thread::sleep(Duration::from_secs(3));
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
                        map_loaded_clone
                            .store(map_downloader_lock.load_chunk(chunk), Ordering::Relaxed);
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

    let mut last_update = Instant::now();
    let mut loading_frame = 0;

    next_frame().await;

    loop {
        clear_background(BLACK);

        if map_loaded.load(Ordering::Relaxed) {
            let map_downloader_lock = map_downloader.lock().unwrap();
            if map.lock().unwrap().is_none()
                && let Some(new_map) = map_downloader_lock.try_build_map()
            {
                if !texture_inited {
                    texture_inited = new_map.init_texture();
                }
                *map.lock().unwrap() = Some(Arc::new(new_map));
            }
        } else {
            thread::sleep(Duration::from_millis(50));
        }

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
                map_arc.render((player.x, player.y));
            }

            gs_arc.render(current_id, (player.x, player.y));
        } else {
            if last_update.elapsed() > std::time::Duration::from_millis(300) {
                loading_frame += 1;
                last_update = std::time::Instant::now();
            }

            loading::draw_loading_screen(loading_frame, &map_downloader);
        }

        next_frame().await;
    }
}
