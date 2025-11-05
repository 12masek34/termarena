use tokio::{net::{TcpListener, TcpStream}, io::AsyncWriteExt};
use crate::utils;
use crate::map::Map;


pub async fn run_server(port: &str) -> Result<String, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    // let extermal_id = utils::get_extermal_id().await?;
    let extermal_id = utils::get_local_ip()
        .await
        .ok_or("Не удалось определить локальный ip")?;
    println!("Сервер слушает на {}:{}", extermal_id, port);

    let map = Map::new(40, 40);
    println!("Карта создана");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Новый игрок подключился: {}", addr);

        let encoded = bincode::serialize(&map)?;
        let len = encoded.len() as u32;

        socket.write_all(&len.to_be_bytes()).await?;
        socket.write_all(&encoded).await?;
        println!("Карта отправлена клиенту {:?} байт", len);
    }
}
