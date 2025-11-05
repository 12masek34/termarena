pub mod ui;
use tokio::io::AsyncReadExt;
use tokio::{net::TcpStream, io::AsyncBufReadExt};
use crate::map::Map;



pub async fn run_client(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(addr).await?;
    println!("Подключено к серверу {}", addr);

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).await?;

    let map: Map = bincode::deserialize(&data)?;
    println!("Карта получена от сервера {}x{}", map.width, map.height);

    ui::render_map(&map);

    Ok(())
}
