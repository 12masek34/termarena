use crate::map::Map;
use crate::{network, utils};
use ::std::sync::Arc;
use tokio::sync::broadcast;
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
    println!("Карта создана");

    let (tx, _rx) = broadcast::channel::<String>(16);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Новый игрок подключился: {}", addr);

        let map_clone = Arc::clone(&map);
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket, map_clone, tx_clone).await {
                eprintln!("Ошибка при обработке клиента {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    socket: &mut tokio::net::TcpStream,
    map: Arc<Map>,
    _tx: tokio::sync::broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    network::send_map(socket, &*map).await?;

    loop {
        // Можно здесь делать рассылку состояния или просто спать
        // чтобы не нагружать CPU
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
