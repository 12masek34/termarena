use tokio::net::TcpStream;


pub async fn run_client(addr: &str) {
    match TcpStream::connect(addr).await {
        Ok(_) => println!("Подключено к серверу {}", addr),
        Err(e) => eprintln!("Не удалось подключиться: {}", e),
    }
}
