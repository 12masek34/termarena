pub mod key_event_handler;
pub mod ui;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::game::state::{GameState, Player};
use crate::map::Map;
use crate::network::{ServerMessage, receive_message};
use futures::FutureExt;
use tokio::net::TcpStream;
use tokio::signal;

#[derive(Debug)]
pub struct ClientState {
    pub id: Option<u32>,
    pub map: Option<Map>,
    pub players: HashMap<u32, Player>,
    pub running: bool,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            id: None,
            map: None,
            players: HashMap::new(),
            running: true,
        }
    }

    pub fn set_map(&mut self, map: Map) {
        self.map = Some(map);
    }

    pub fn set_player(&mut self, player: Player) {
        self.id = Some(player.id)
    }

    pub fn set_game_state(&mut self, game_state: GameState) {
        self.players = game_state.players;
    }
}

pub async fn run_client(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Ошибка подключения к серверу {}: {}", addr, e);
            return Err(Box::new(e));
        }
    };

    let mut state = ClientState::new();
    run_game_loop(&mut state, stream).await;

    Ok(())
}

pub async fn run_game_loop(state: &mut ClientState, stream: TcpStream) {
    // ui::start_game_screen();
    let (reader, writer) = tokio::io::split(stream);
    let reader = Arc::new(Mutex::new(reader));
    let writer = Arc::new(Mutex::new(writer));
    let mut input_task: Option<JoinHandle<()>> = None;

    while state.running {
        tokio::select! {
            res = async {
                let mut reader_guard = reader.lock().await;
                receive_message::<ServerMessage>(&mut *reader_guard).await
            }.fuse() => {
                match res {
                    Ok(message) => {
                        match message {
                            ServerMessage::Map(map) => state.set_map(map),
                            ServerMessage::InitPlayer(player) => {
                                let player_id: u32 = player.id;
                                state.set_player(player);

                                if input_task.is_none() {
                                    let writer_clone = Arc::clone(&writer);
                                    input_task = Some(tokio::spawn(async move {
                                        let _ = key_event_handler::handle_input(writer_clone, player_id).await;
                                    }));
                                }
                            },
                            ServerMessage::GameState(game_state) => state.set_game_state(game_state),
                        }
                    },
                    Err(e) => {
                        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                            if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                                println!("Сервер закрыл соединение");
                                state.running = false;
                                break;
                            }
                        }
                        eprintln!("Ошибка приёма сообщения: {}", e);
                    }
                }
            },
            _ = signal::ctrl_c() => {
                state.running = false;
            }
        }
    }
    if let Some(handle) = input_task.take() {
        handle.abort();
    }
    // ui::end_game_screen();
}
