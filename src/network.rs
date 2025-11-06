use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::game::state::Player;
use crate::{game::state::GameState, map::Map};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Map(Map),
    Player(Player),
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

pub async fn send_init_state(
    socket: &mut tokio::net::TcpStream,
    map: &Map,
    game_state: &Mutex<GameState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let server_message = ServerMessage::Map(map.clone());
    send_data(socket, &server_message).await?;
    println!(
        "Карта отправлена клиенту ({:?} байт)",
        bincode::serialized_size(&*map)?
    );

    let mut state = game_state.lock().await;
    let player = state.create_player(&map);
    let server_message = ServerMessage::Player(player);
    send_data(socket, &server_message).await?;
    println!(
        "Пользователь отправлен ({:?} байт)",
        bincode::serialized_size(&*map)?
    );

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
