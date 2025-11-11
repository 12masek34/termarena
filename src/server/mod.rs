use std::{
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        mpsc::{self},
    },
    thread,
};

use crate::{
    game::state::GameState,
    map::Map,
    network::{
        recv_message, send_message,
        state::{ClientMessage, ServerMessage},
    },
};

type SharedGameState = Arc<Mutex<GameState>>;
type SharedClients = Arc<Mutex<Vec<SocketAddr>>>;

pub fn run_server(port: String) {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).expect("Could not bind UDP socket");
    socket
        .set_nonblocking(false)
        .expect("Failed to set blocking mode");

    let map = Map::new(100, 100);
    let game_state: SharedGameState = Arc::new(Mutex::new(GameState::new()));
    let clients: SharedClients = Arc::new(Mutex::new(Vec::new()));
    let mut player_id: Option<u32> = None;

    println!("Server running on port {}", port);

    let (tx, rx) = mpsc::channel::<ServerMessage>();

    let clients_clone = Arc::clone(&clients);
    let socket_clone = socket.try_clone().unwrap();

    thread::spawn(move || {
        for msg in rx {
            let clients_guard = clients_clone.lock().unwrap();
            for &client in clients_guard.iter() {
                let _ = send_message(&socket_clone, &msg, client);
            }
        }
    });

    loop {
        if let Some((msg, src)) = recv_message::<ClientMessage>(&socket) {
            let mut clients_guard = clients.lock().unwrap();

            if !clients_guard.contains(&src) {
                clients_guard.push(src);
                println!("New client: {}", src);
            }

            match msg {
                ClientMessage::Init => {
                    println!("Player init");
                    let player = game_state.lock().unwrap().create_player(&map);
                    player_id = Some(player.id);
                    let _ = tx
                        .send(ServerMessage::Map(map.clone()))
                        .expect("failed to send to net thread");
                    let _ = tx
                        .send(ServerMessage::InitPlayer(player))
                        .expect("failed to send to net thread");
                    let _ = tx
                        .send(ServerMessage::GameState(game_state.lock().unwrap().clone()))
                        .expect("failed to send to net thread");
                }
                ClientMessage::Move(x, y) => {
                    game_state
                        .lock()
                        .unwrap()
                        .move_player(player_id, x, y, &map);
                    let _ = tx.send(ServerMessage::GameState(game_state.lock().unwrap().clone()));
                }
                ClientMessage::Quit => {
                    println!("Player disconnected");
                }
            }
        }
    }
}
