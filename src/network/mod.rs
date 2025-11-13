pub mod state;
use crate::config;
use crate::map::Map;
use serde::{Serialize, de::DeserializeOwned};
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
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

pub fn send_map_to_client(map: &Map, client_addr: std::net::SocketAddr) {
    let addr = format!("{}:{}", client_addr.ip(), config::TCP_PORT);
    match TcpStream::connect(addr) {
        Ok(mut stream) => {
            let data = bincode::serialize(map).expect("Failed to serialize map");
            let size = data.len() as u64;
            println!("Sending map to {}: {} bytes", client_addr, size);
            stream
                .write_all(&size.to_be_bytes())
                .expect("Failed to send size");
            stream.write_all(&data).expect("Failed to send map");
            println!("Map sent to client {}", client_addr);
        }
        Err(e) => eprintln!("Failed to connect to client {}: {:?}", client_addr, e),
    }
}

pub fn get_map_from_tcp() -> Option<Map> {
    let listener =
        TcpListener::bind(("0.0.0.0", config::TCP_PORT as u16)).expect("Failed to bind TCP port");
    println!("TCP map listener running on port {}", config::TCP_PORT);
    if let Ok((mut stream, addr)) = listener.accept() {
        println!("Incoming TCP connection from {}", addr);
        let mut size_buf = [0u8; 8];
        stream
            .read_exact(&mut size_buf)
            .expect("Failed to read size");
        let size = u64::from_be_bytes(size_buf) as usize;
        let mut data = vec![0u8; size];
        stream.read_exact(&mut data).expect("Failed to read map");
        let map: Map = bincode::deserialize(&data).expect("Failed to deserialize map");
        println!("Map received via TCP");

        return Some(map);
    }
    println!("TCP map listener finished");
    None
}
