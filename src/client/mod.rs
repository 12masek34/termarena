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
        if self.map.is_none() {
            self.map = Some(map);
        }
    }

    pub fn set_player(&mut self, player: Player) {
        self.id = Some(player.id)
    }

    pub fn set_game_state(&mut self, game_state: GameState) {
        self.players = game_state.players;
    }

    pub fn current_player(&self) -> Option<&Player> {
        self.id.and_then(|id| self.players.get(&id))
    }

    pub fn build_frame(&self, viewport_width: usize, viewport_height: usize) -> String {
        let map = match &self.map {
            Some(m) => m,
            None => return String::new(),
        };

        let player = match self.current_player() {
            Some(p) => p,
            None => return String::new(),
        };

        let half_w = viewport_width / 2;
        let half_h = viewport_height / 2;

        let min_x = player.x.saturating_sub(half_w);
        let min_y = player.y.saturating_sub(half_h);

        let max_x = (min_x + viewport_width).min(map.width);
        let max_y = (min_y + viewport_height).min(map.height);

        let mut frame_rows: Vec<String> = Vec::with_capacity(viewport_height);

        for y in min_y..max_y {
            let mut row_str = String::new();
            for x in min_x..max_x {
                let mut ch = map.tiles[y][x];

                for (_id, other) in self.players.iter() {
                    if other.x == x && other.y == y {
                        ch = '@';
                        break;
                    }
                }

                row_str.push(ch);
            }
            frame_rows.push(row_str);
        }

        let mut frame = frame_rows.join("\n");

        frame.push_str(&format!(
            "\nYou: {}\nPosition: ({},{})\nPlayers nearby: {}\nMap size: {}x{}\n",
            self.id.unwrap_or(0),
            player.x,
            player.y,
            self.players.len(),
            map.width,
            map.height,
        ));

        frame
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

    let state = ClientState::new();
    run_game_loop(state, stream).await;

    Ok(())
}

pub async fn run_game_loop(state: ClientState, stream: TcpStream) {
    let _raw_mode_guard = scopeguard::guard((), |_| {
        let _ = crossterm::terminal::disable_raw_mode();
        println!("Сервер разорвал соединение");
    });

    ui::start_game_screen();
    let state = Arc::new(Mutex::new(state));
    let (reader, writer) = tokio::io::split(stream);
    let reader = Arc::new(Mutex::new(reader));
    let writer = Arc::new(Mutex::new(writer));

    let mut input_task: Option<JoinHandle<()>> = None;

    loop {
        let running = { state.lock().await.running };
        if !running {
            break;
        }

        tokio::select! {
            res = async {
                let mut reader_guard = reader.lock().await;
                receive_message::<ServerMessage>(&mut *reader_guard).await
            }.fuse() => {
                match res {
                    Ok(message) => {
                        let mut state_guard = state.lock().await;
                        match message {
                            ServerMessage::Map(map) => state_guard.set_map(map),
                            ServerMessage::InitPlayer(player) => {
                                state_guard.set_player(player);
                                if input_task.is_none() {
                                    let writer_clone = Arc::clone(&writer);
                                    let state_clone = Arc::clone(&state);
                                    input_task = Some(tokio::spawn(async move {
                                        let _ = key_event_handler::handle_input(state_clone, writer_clone).await;
                                    }));
                                }
                            },
                            ServerMessage::GameState(game_state) => state_guard.set_game_state(game_state),
                        }
                        let _ = ui::render(&*state_guard);
                    },
                    Err(e) => {
                        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                            if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                                state.lock().await.running = false;
                                break;
                            }
                        }
                        eprintln!("Ошибка приёма сообщения: {}", e);
                    }
                }
            },

            _ = signal::ctrl_c() => {
                state.lock().await.running = false;
            },

            Some(_) = async {
                if let Some(handle) = &mut input_task {
                    Some(handle.await.ok())
                } else { None }
            } => {
                state.lock().await.running = false;
            }
        }
    }

    if let Some(handle) = input_task.take() {
        handle.abort();
    }
    ui::end_game_screen();
}
