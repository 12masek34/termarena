use std::sync::Arc;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::{
    game::state::{Direction, GameState, Player},
    map::Map,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Map(Map),
    InitPlayer(Player),
    GameState(GameState),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Move(Direction),
    Quit,
}

pub async fn send_init_state(
    socket: &Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
    map: &Map,
    game_state: &Mutex<GameState>,
    tx: &tokio::sync::broadcast::Sender<ServerMessage>,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut socket = socket.lock().await;
    let server_message = ServerMessage::Map(map.clone());
    send_message(&mut socket, &server_message).await?;
    let mut state = game_state.lock().await;
    let new_player = state.create_player(&map);
    let id = new_player.id;
    let server_message = ServerMessage::InitPlayer(new_player);
    send_message(&mut socket, &server_message).await?;
    tx.send(ServerMessage::GameState(state.clone()))?;

    Ok(id)
}

pub async fn send_message<T>(
    stream: &mut tokio::io::WriteHalf<TcpStream>,
    message: &T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let data = bincode::serialize(message)?;
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&data).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn receive_message<T>(
    stream: &mut tokio::io::ReadHalf<TcpStream>,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    let message: T = bincode::deserialize(&buf)?;
    Ok(message)
}
