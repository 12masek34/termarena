use std::{
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        mpsc::{self},
    },
    thread,
};

use crate::{
    game::state::{GameState, Player},
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

    let (tx, rx) = mpsc::channel::<()>();

    {
        let gs_clone = Arc::clone(&game_state);
        let clients_clone = Arc::clone(&clients);
        let socket_clone = socket.try_clone().unwrap();
        thread::spawn(move || {
            while rx.recv().is_ok() {
                let gs = gs_clone.lock().unwrap();
                let clients_guard = clients_clone.lock().unwrap();
                for &client in clients_guard.iter() {
                    let _ =
                        send_message(&socket_clone, &ServerMessage::GameState(gs.clone()), client);
                }
            }
        });
    }

    let socket_clone_init = socket.try_clone().unwrap();
    loop {
        if let Some((msg, src)) = recv_message::<ClientMessage>(&socket) {
            let mut clients_guard = clients.lock().unwrap();

            if !clients_guard.contains(&src) {
                clients_guard.push(src);
                println!("New client: {}", src);
                let new_player = game_state.lock().unwrap().create_player();
                send_message(
                    &socket_clone_init,
                    &ServerMessage::InitPlayer(new_player),
                    src,
                );
            }

            let mut gs = game_state.lock().unwrap();
            match msg {
                ClientMessage::Init => {
                    println!("Player init");
                }
                ClientMessage::Quit => {
                    println!("Player disconnected");
                }
                ClientMessage::Move(x, y) => {
                    println!("Move");
                }
            }

            let _ = tx.send(());
        }
    }
}
