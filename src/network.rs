use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::map::Map;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Map(Map),
}

pub async fn send_data<T: serde::Serialize>(
    socket: &mut tokio::net::TcpStream,
    data: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = bincode::serialize(data)?;
    let len = encoded.len() as u32;

    socket.write_all(&len.to_be_bytes()).await?;
    socket.write_all(&encoded).await?;
    Ok(())
}

pub async fn send_map(
    socket: &mut tokio::net::TcpStream,
    map: &Map,
) -> Result<(), Box<dyn std::error::Error>> {
    let server_message = ServerMessage::Map(map.clone());
    send_data(socket, &server_message).await?;

    Ok(())
}

pub async fn receive_message(
    stream: &mut TcpStream,
) -> Result<ServerMessage, Box<dyn std::error::Error>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).await?;
    let server_message: ServerMessage = bincode::deserialize(&data)?;

    Ok(server_message)
}

// pub async fn receive_map(stream: &mut TcpStream) -> Result<Map, Box<dyn std::error::Error>> {
//     let data = receive_data(stream).await?;
//     let server_message: ServerMessage = bincode::deserialize(&data)?;

//     match server_message {
//         ServerMessage::Map(map) => Ok(map),
//         _ => Err("Неизвестный тип сообщения от сервера".into()),
//     }
// }
