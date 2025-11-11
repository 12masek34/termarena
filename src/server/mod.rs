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

    let game_state: SharedGameState = Arc::new(Mutex::new(GameState::new()));
    let clients: SharedClients = Arc::new(Mutex::new(Vec::new()));

    println!("Server running on port {}", port);

    let (tx, rx) = mpsc::channel::<(ServerMessage, SocketAddr)>();

    {
        let socket_clone = socket.try_clone().unwrap();
        thread::spawn(move || {
            for (msg, target) in rx {
                send_message(&socket_clone, &msg, target);
            }
        });
    }

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
                    let player = game_state.lock().unwrap().create_player();
                    tx.send((ServerMessage::Map, src))
                        .expect("failed to send to net thread");
                    tx.send((ServerMessage::InitPlayer(player), src))
                        .expect("failed to send to net thread");
                }
                ClientMessage::Quit => {
                    println!("Player disconnected");
                }
                ClientMessage::Move(x, y) => {
                    println!("Move");
                    tx.send((
                        ServerMessage::GameState(game_state.lock().unwrap().clone()),
                        src,
                    ));
                }
            }
        }
    }
}
