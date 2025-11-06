pub mod ui;

use std::fmt::Display;

use crate::game::state::Player;
use crate::map::Map;
use crate::network::{ServerMessage, receive_message};
use tokio::net::TcpStream;
use tokio::signal;

pub struct ClientState {
    pub map: Option<Map>,
    pub player_x: usize,
    pub player_y: usize,
    pub running: bool,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            map: None,
            player_x: 0,
            player_y: 0,
            running: true,
        }
    }

    pub fn set_map(&mut self, map: Map) {
        self.map = Some(map);
    }

    pub fn set_player(&mut self, player: Player) {
        self.player_x = player.x;
        self.player_y = player.y;
    }
}

pub async fn run_client(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Ошибка подключения к серверу {}: {}", addr, e);
            return Err(Box::new(e));
        }
    };

    let mut state = ClientState::new();
    run_game_loop(&mut state, &mut stream).await;

    Ok(())
}

pub async fn run_game_loop(state: &mut ClientState, stream: &mut TcpStream) {
    ui::start_game_screen();

    while state.running {
        tokio::select! {
            res = receive_message(stream) => {
                match res {
                    Ok(message) => {
                        match message {
                            ServerMessage::Map(map) => {
                                state.set_map(map);
                            },
                            ServerMessage::Player(player) => {
                                state.set_player(player);
                            },
                            // Добавляем обработку других типов сообщений
                            // ServerMessage::Chat(msg) => { ... }
                            // ServerMessage::PlayerUpdate(p) => { ... }
                        }

                        ui::render(state);
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

    ui::end_game_screen();
}
