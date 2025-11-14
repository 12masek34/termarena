pub mod state;
use serde::{Serialize, de::DeserializeOwned};
use std::net::SocketAddr;
use std::net::UdpSocket;

pub fn recv_message<T: DeserializeOwned>(socket: &UdpSocket) -> Option<(T, SocketAddr)> {
    let mut buf = [0u8; 65536];
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => match bincode::deserialize::<T>(&buf[..amt]) {
            Ok(msg) => Some((msg, src)),
            Err(e) => {
                eprintln!("Failed to deserialize message: {:?}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to receive from socket: {:?}", e);
            None
        }
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
