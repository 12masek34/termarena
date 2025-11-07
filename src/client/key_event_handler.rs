use std::sync::Arc;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

use crate::game::state;
use crate::network::{ClientMessage, send_message};

pub async fn handle_input(
    stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    player_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    loop {
        let ev = tokio::task::spawn_blocking(|| event::read()).await??;

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = ev
        {
            if (code == KeyCode::Char('q') && modifiers.is_empty())
                || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
            {
                println!("🚪 Выход");
                break;
            }

            let direction = match code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    println!("⬆️ Вверх");
                    Some(state::Direction::Up)
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    println!("⬇️ Вниз");
                    Some(state::Direction::Down)
                }
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                    println!("⬅️ Влево");
                    Some(state::Direction::Left)
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                    println!("➡️ Вправо");
                    Some(state::Direction::Right)
                }
                _ => None,
            };

            if let Some(dir) = direction {
                let stream_clone = Arc::clone(&stream);
                tokio::spawn(async move {
                    let mut sock = stream_clone.lock().await;
                    if let Err(e) = send_message(&mut *sock, &ClientMessage::Move(dir)).await {
                        eprintln!("Ошибка при отправке сообщения клиенту: {e}");
                    };
                });
            }
        }
    }
    disable_raw_mode()?;

    Ok(())
}
