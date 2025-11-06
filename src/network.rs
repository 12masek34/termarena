use std::{mem::replace, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::{
    game::state::{GameState, Player},
    map::Map,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Map(Map),
    InitPlayer(Player),
    GameState(GameState),
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
    socket: &Arc<Mutex<tokio::net::TcpStream>>,
    map: &Map,
    game_state: &Mutex<GameState>,
    tx: &tokio::sync::broadcast::Sender<ServerMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = socket.lock().await;
    let server_message = ServerMessage::Map(map.clone());
    send_data(&mut socket, &server_message).await?;
    let mut state = game_state.lock().await;
    let new_player = state.create_player(&map);
    let server_message = ServerMessage::InitPlayer(new_player);
    send_data(&mut socket, &server_message).await?;
    tx.send(ServerMessage::GameState(state.clone()))?;

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
