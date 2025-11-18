use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        mpsc::{self},
    },
    thread,
    time::{Duration, Instant},
};

use crate::{config, network::state::ServerMessageType};
use crate::{
    game::state::GameState,
    map::Map,
    network::{
        recv_message, send_message,
        state::{ClientMessage, ServerMessage},
    },
};

type SharedGameState = Arc<Mutex<GameState>>;
type SharedClients = Arc<Mutex<HashMap<SocketAddr, u32>>>;

pub fn run_server(port: String) {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).expect("Could not bind UDP socket");
    socket
        .set_nonblocking(false)
        .expect("Failed to set blocking mode");

    let map = Arc::new(Map::new(config::MAP_WIDTH, config::MAP_HEIGHT));
    let game_state: SharedGameState = Arc::new(Mutex::new(GameState::new()));
    let clients: SharedClients = Arc::new(Mutex::new(HashMap::new()));
    println!("Server running on port {}", port);

    let (tx, rx) = mpsc::channel::<ServerMessage>();

    let clients_clone = Arc::clone(&clients);
    let socket_clone = socket.try_clone().unwrap();
    thread::spawn(move || {
        for msg in rx {
            let clients_snapshot = {
                let clients_guard = clients_clone.lock().unwrap();
                clients_guard.clone()
            };
            for (&client, _) in &clients_snapshot {
                let _ = send_message(&socket_clone, &msg, client);
            }
        }
    });

    let game_state_clone = Arc::clone(&game_state);
    let clients_clone_gs = Arc::clone(&clients);
    let map_clone = Arc::clone(&map);
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let tick_rate = Duration::from_millis(30);
        let mut last_update = Instant::now();
        loop {
            let start = Instant::now();
            let clients_snapshot = {
                let clients_guard = clients_clone_gs.lock().unwrap();
                clients_guard.clone()
            };
            {
                let mut game_state_lock = game_state_clone.lock().unwrap();
                let now = Instant::now();
                let delta_time = (now - last_update).as_secs_f32();
                last_update = now;
                game_state_lock.update(&map_clone, delta_time);

                for (&src, _player_id) in &clients_snapshot {
                    let snapshot_diff = game_state_lock.get_snapshot_diff();
                    let _ = tx_clone
                        .send(ServerMessage {
                            src: src,
                            message: ServerMessageType::GameStateDiff(snapshot_diff),
                        })
                        .expect("failed to send to net thread");
                }
            }
            let elapsed = start.elapsed();

            if elapsed < tick_rate {
                thread::sleep(tick_rate - elapsed);
            }
        }
    });

    loop {
        if let Some((msg, src)) = recv_message::<ClientMessage>(&socket) {
            {
                let mut clients_guard = clients.lock().unwrap();
                if !clients_guard.contains_key(&src) {
                    clients_guard.insert(src, 0);
                    println!("New client: {}", src);
                }
            }

            match msg {
                ClientMessage::Init => {
                    println!("Player init");
                    let player = game_state.lock().unwrap().create_player(&map);

                    {
                        let mut clients_lock = clients.lock().unwrap();
                        clients_lock.insert(src, player.id);
                    }
                    let _ = tx
                        .send(ServerMessage {
                            src: src,
                            message: ServerMessageType::InitPlayer(player),
                        })
                        .expect("failed to send to net thread");

                    let snapshot = {
                        let mut game_state_lock = game_state.lock().unwrap();
                        game_state_lock.get_snapshot()
                    };
                    let _ = tx
                        .send(ServerMessage {
                            src: src,
                            message: ServerMessageType::GameState(snapshot),
                        })
                        .expect("failed to send to net thread");
                }
                ClientMessage::Map(chunk_ids) => {
                    let chunks = map.chunk_map();
                    for chunk in chunks {
                        if !chunk_ids.contains(&chunk.chunk_index) {
                            tx.send(ServerMessage {
                                src: src,
                                message: ServerMessageType::Map(chunk),
                            })
                            .expect("failed to send to net thread");
                        }
                    }
                }
                ClientMessage::Move(direction) => {
                    {
                        let clients_lock = clients.lock().unwrap();
                        let player_id = clients_lock.get(&src);
                        let mut game_state_lock = game_state.lock().unwrap();
                        game_state_lock.move_player(player_id, direction, &map);
                    }
                    let snapshot_diff = {
                        let mut game_state = game_state.lock().unwrap();
                        game_state.get_snapshot_diff()
                    };
                    let _ = tx
                        .send(ServerMessage {
                            src: src,
                            message: ServerMessageType::GameStateDiff(snapshot_diff),
                        })
                        .expect("failed to send to net thread");
                }
                ClientMessage::Shoot => {
                    {
                        let clients_lock = clients.lock().unwrap();
                        let player_id = clients_lock.get(&src);
                        let mut game_state_lock = game_state.lock().unwrap();
                        game_state_lock.shoot(player_id);
                    }
                    let snapshot = {
                        let mut game_state = game_state.lock().unwrap();
                        game_state.get_snapshot()
                    };
                    let _ = tx
                        .send(ServerMessage {
                            src: src,
                            message: ServerMessageType::GameState(snapshot),
                        })
                        .expect("failed to send to net thread");
                }
                ClientMessage::Quit => {
                    println!("Player disconnected {}", src);
                    {
                        let mut clients_lock = clients.lock().unwrap();
                        let player_id = clients_lock.get(&src);
                        let mut game_state_lock = game_state.lock().unwrap();
                        game_state_lock.remove(player_id);
                        clients_lock.remove(&src);
                    }
                    let snapshot = {
                        let mut game_state = game_state.lock().unwrap();
                        game_state.get_snapshot()
                    };
                    let _ = tx
                        .send(ServerMessage {
                            src: src,
                            message: ServerMessageType::GameState(snapshot),
                        })
                        .expect("failed to send to net thread");
                }
            }
        }
    }
}
