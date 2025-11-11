pub mod state;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_PACKET_SIZE: usize = 1200;
const MSG_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Serialize, Deserialize, Debug)]
struct Packet {
    header: ChunkHeader,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChunkHeader {
    msg_id: u64,
    chunk_index: u32,
    total_chunks: u32,
}

// Глобальный сборщик сообщений
lazy_static::lazy_static! {
    static ref MSG_ASSEMBLER: Mutex<MessageAssembler> = Mutex::new(MessageAssembler::new());
}

struct MessageAssembler {
    partial: HashMap<u64, (Instant, Vec<Option<Vec<u8>>>, SocketAddr)>,
}

impl MessageAssembler {
    fn new() -> Self {
        Self {
            partial: HashMap::new(),
        }
    }

    fn add_chunk<T: DeserializeOwned>(
        &mut self,
        header: ChunkHeader,
        data: Vec<u8>,
        src: SocketAddr,
    ) -> Option<(T, SocketAddr)> {
        let entry = self.partial.entry(header.msg_id).or_insert_with(|| {
            (
                Instant::now(),
                vec![None; header.total_chunks as usize],
                src,
            )
        });

        if header.chunk_index as usize >= entry.1.len() {
            eprintln!("Chunk index out of range");
            return None;
        }

        entry.1[header.chunk_index as usize] = Some(data);

        if entry.1.iter().all(|c| c.is_some()) {
            let mut full_data = Vec::new();
            for part in entry.1.iter().flatten() {
                full_data.extend_from_slice(part);
            }
            self.partial.remove(&header.msg_id);
            match deserialize::<T>(&full_data) {
                Ok(msg) => return Some((msg, src)),
                Err(e) => eprintln!("Failed to deserialize full message: {:?}", e),
            }
        }
        self.partial
            .retain(|_, (t, _, _)| t.elapsed() < MSG_TIMEOUT);

        None
    }
}

pub fn send_message<T: Serialize>(socket: &UdpSocket, msg: &T, target: SocketAddr) -> bool {
    match serialize(msg) {
        Ok(data) => {
            let total_chunks = ((data.len() + MAX_PACKET_SIZE - 1) / MAX_PACKET_SIZE) as u32;
            let msg_id = rand::random::<u64>();

            for (i, chunk) in data.chunks(MAX_PACKET_SIZE).enumerate() {
                let packet = Packet {
                    header: ChunkHeader {
                        msg_id,
                        chunk_index: i as u32,
                        total_chunks,
                    },
                    data: chunk.to_vec(),
                };

                let packet_bytes = match serialize(&packet) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Failed to serialize packet {}: {:?}", i, e);
                        return false;
                    }
                };

                if let Err(e) = socket.send_to(&packet_bytes, target) {
                    eprintln!("Failed to send packet {}: {:?}", i, e);
                    return false;
                }
            }

            true
        }
        Err(e) => {
            eprintln!("Failed to serialize message: {:?}", e);
            false
        }
    }
}

pub fn recv_message<T: DeserializeOwned>(socket: &UdpSocket) -> Option<(T, SocketAddr)> {
    let mut buf = [0u8; 65536];
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => match deserialize::<Packet>(&buf[..amt]) {
            Ok(packet) => {
                MSG_ASSEMBLER
                    .lock()
                    .unwrap()
                    .add_chunk::<T>(packet.header, packet.data, src)
            }
            Err(_) => match deserialize::<T>(&buf[..amt]) {
                Ok(msg) => Some((msg, src)),
                Err(e) => {
                    eprintln!("Failed to deserialize message: {:?}", e);
                    None
                }
            },
        },
        Err(e) => {
            eprintln!("Failed to receive from socket: {:?}", e);
            None
        }
    }
}
