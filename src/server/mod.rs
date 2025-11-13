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

use crate::config;
use crate::{
    game::state::GameState,
    map::Map,
    network::{
        recv_message, send_map_to_client, send_message,
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
    let map_clone = Arc::clone(&map);
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let tick_rate = Duration::from_millis(50);
        loop {
            let start = Instant::now();
            let snapshot = {
                let mut game_state_lock = game_state_clone.lock().unwrap();
                game_state_lock.update(&map_clone);
                game_state_lock.get_snapshot()
            };
            let _ = tx_clone.send(ServerMessage::GameState(snapshot));
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
                    send_map_to_client(&map, src);
                    let _ = tx
                        .send(ServerMessage::InitPlayer(player))
                        .expect("failed to send to net thread");
                }
                ClientMessage::Move(direction) => {
                    {
                        let clients_lock = clients.lock().unwrap();
                        let player_id = clients_lock.get(&src);
                        let mut game_state_lock = game_state.lock().unwrap();
                        game_state_lock.move_player(player_id, direction, &map);
                    }
                    let snapshot = {
                        let game_state = game_state.lock().unwrap();
                        game_state.get_snapshot()
                    };
                    let _ = tx
                        .send(ServerMessage::GameState(snapshot))
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
                        let game_state = game_state.lock().unwrap();
                        game_state.get_snapshot()
                    };
                    let _ = tx
                        .send(ServerMessage::GameState(snapshot))
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
                        let game_state = game_state.lock().unwrap();
                        game_state.get_snapshot()
                    };
                    let _ = tx
                        .send(ServerMessage::GameState(snapshot))
                        .expect("failed to send to net thread");
                }
            }
        }
    }
}
