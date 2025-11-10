pub mod state;
use std::net::SocketAddr;
use std::net::UdpSocket;

use serde::Serialize;
use state::{ClientMessage, ServerMessage};

pub fn recv_message(socket: &UdpSocket) -> Option<(ClientMessage, SocketAddr)> {
    let mut buf = [0u8; 1024];
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => match bincode::deserialize::<ClientMessage>(&buf[..amt]) {
            Ok(msg) => Some((msg, src)),
            Err(e) => {
                eprintln!("Failed to deserialize ClientMessage: {:?}", e);
                None
            }
        },
        Err(_) => None,
    }
}

pub fn send_message<T: Serialize>(socket: &UdpSocket, msg: &T, target: SocketAddr) -> bool {
    match bincode::serialize(msg) {
        Ok(data) => match socket.send_to(&data, target) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to send message: {:?}", e);
                false
            }
        },
        Err(e) => {
            eprintln!("Failed to serialize message: {:?}", e);
            false
        }
    }
}
