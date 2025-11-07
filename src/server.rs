use crate::game::state::GameState;
use crate::map::Map;
use crate::network::{ClientMessage, ServerMessage, receive_message, send_message};
use crate::{network, utils};
use ::std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};

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

    let map = Arc::new(Map::new(100, 100));
    let game_state = Arc::new(Mutex::new(GameState::new()));
    println!("Карта создана");

    let (tx, _rx) = broadcast::channel::<ServerMessage>(100);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Новый игрок подключился: {}", addr);

        let map_clone = Arc::clone(&map);
        let game_state_clone = Arc::clone(&game_state);
        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, map_clone, game_state_clone, tx, rx).await {
                eprintln!("Ошибка при обработке клиента {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    map: Arc<Map>,
    game_state: Arc<Mutex<GameState>>,
    tx: tokio::sync::broadcast::Sender<ServerMessage>,
    mut rx: tokio::sync::broadcast::Receiver<ServerMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = tokio::io::split(socket);
    let reader = Arc::new(Mutex::new(reader));
    let writer = Arc::new(Mutex::new(writer));

    let player_id = network::send_init_state(&writer, &*map, &*game_state, &tx).await?;
    tokio::spawn(async move {
        loop {
            if let Ok(server_message) = rx.recv().await {
                let mut write_guard = writer.lock().await;
                if let Err(e) = send_message(&mut *write_guard, &server_message).await {
                    eprintln!("Ошибка при отправке сообщения клиенту: {:?}", e);
                    break;
                }
            }
        }
    });

    loop {
        let client_message = {
            let mut reader_guard = reader.lock().await;
            match receive_message(&mut *reader_guard).await {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Ошибка при получении сообщения от клиента: {:?}", e);
                    break;
                }
            }
        };
        match client_message {
            ClientMessage::Move(direction) => {
                let mut state_guard = game_state.lock().await;
                state_guard.move_player(player_id, direction, &map);
            }
            ClientMessage::Quit => {
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!111");
                break;
            }
        }
        let state_guard = game_state.lock().await;
        tx.send(ServerMessage::GameState(state_guard.clone()))?;
    }

    Ok(())
}
