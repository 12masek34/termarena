use std::sync::Arc;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

use crate::game::state;
use crate::network::{ClientMessage, send_message};

use super::ClientState;

pub async fn handle_input(
    state: Arc<Mutex<ClientState>>,
    stream: Arc<Mutex<WriteHalf<TcpStream>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    loop {
        let ev = tokio::task::spawn_blocking(|| event::read()).await??;

        let message = if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = ev
        {
            match (code, modifiers) {
                (KeyCode::Up, _)
                | (KeyCode::Char('w'), _)
                | (KeyCode::Char('W'), _)
                | (KeyCode::Char('k'), _)
                | (KeyCode::Char('K'), _) => Some(ClientMessage::Move(state::Direction::Up)),
                (KeyCode::Down, _)
                | (KeyCode::Char('s'), _)
                | (KeyCode::Char('S'), _)
                | (KeyCode::Char('j'), _)
                | (KeyCode::Char('J'), _) => Some(ClientMessage::Move(state::Direction::Down)),
                (KeyCode::Left, _)
                | (KeyCode::Char('a'), _)
                | (KeyCode::Char('A'), _)
                | (KeyCode::Char('h'), _)
                | (KeyCode::Char('H'), _) => Some(ClientMessage::Move(state::Direction::Left)),
                (KeyCode::Right, _)
                | (KeyCode::Char('d'), _)
                | (KeyCode::Char('D'), _)
                | (KeyCode::Char('l'), _)
                | (KeyCode::Char('L'), _) => Some(ClientMessage::Move(state::Direction::Right)),
                (KeyCode::Char('q'), _) => Some(ClientMessage::Quit),
                (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                    Some(ClientMessage::Quit)
                }
                _ => None,
            }
        } else {
            None
        };

        if let Some(msg) = message {
            let stream_clone = Arc::clone(&stream);
            let msg_clone = msg.clone();
            tokio::spawn(async move {
                let mut sock = stream_clone.lock().await;
                if let Err(e) = send_message(&mut *sock, &msg_clone).await {
                    eprintln!("Ошибка при отправке сообщения серверу: {e}");
                }
            });

            if let ClientMessage::Quit = msg {
                state.lock().await.running = false;
                break;
            }
        }
    }
    disable_raw_mode()?;

    Ok(())
}
