use crate::game::state::GameState;
use crate::map::Map;
use crate::network::{ServerMessage, send_data};
use crate::{network, utils};
use ::std::sync::Arc;
use serde::Serialize;
use std::net::SocketAddr;
use tokio::sync::{Mutex, broadcast};
use tokio::time::{Duration, sleep};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

pub async fn run_server(port: &str) -> Result<String, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let extermal_id = utils::get_extermal_id().await?;
    let internal_id = utils::get_local_ip()
        .await
        .ok_or("Не удалось определить локальный ip")?;
    println!(
        "Сервер запущен:\nвнутренний адрес {}:{}\nнаружний адрес {}:{}",
        extermal_id, port, internal_id, port
    );

    let map = Arc::new(Map::new(40, 40));
    let game_state = Arc::new(Mutex::new(GameState::new()));
    println!("Карта создана");

    let (tx, _rx) = broadcast::channel::<ServerMessage>(100);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        let socket = Arc::new(Mutex::new(socket));
        println!("Новый игрок подключился: {}", addr);

        let map_clone = Arc::clone(&map);
        let game_state_clone = Arc::clone(&game_state);
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, map_clone, game_state_clone, tx, rx).await {
                eprintln!("Ошибка при обработке клиента {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    socket: Arc<Mutex<tokio::net::TcpStream>>,
    map: Arc<Map>,
    game_state: Arc<Mutex<GameState>>,
    tx: tokio::sync::broadcast::Sender<ServerMessage>,
    mut rx: tokio::sync::broadcast::Receiver<ServerMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    network::send_init_state(&socket, &*map, &*game_state, &tx).await?;

    tokio::spawn(async move {
        loop {
            let mut sock = socket.lock().await;
            if let Ok(server_message) = rx.recv().await {
                if let Err(e) = send_data(&mut *sock, &server_message).await {
                    eprintln!("Ошибка при отправке сообщения клиенту: {:?}", e);
                    break;
                }
            }
        }
    });

    loop {
        // Можно здесь делать рассылку состояния или просто спать
        // чтобы не нагружать CPU
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
