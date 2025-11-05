use tokio::net::TcpListener;
use crate::network;
use crate::utils;


pub async fn run_server(port: &str) -> Result<String, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    // let extermal_id = utils::get_extermal_id().await?;
    let extermal_id = utils::get_local_ip()
        .await
        .ok_or("Не удалось определить локальный ip")?;
    println!("Сервер слушает на {}:{}", extermal_id, port);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Новый игрок подключился: {}", addr);
    }
}
