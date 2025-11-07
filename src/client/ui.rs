use super::ClientState;
use crossterm::{
    ExecutableCommand,
    cursor::{Hide, Show},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{Write, stdout};

pub fn start_game_screen() {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen).unwrap();
    stdout.execute(Hide).unwrap();
}

pub fn end_game_screen() {
    let mut stdout = stdout();
    stdout.execute(Show).unwrap();
    stdout.execute(LeaveAlternateScreen).unwrap();
}

pub fn render(state: &ClientState) -> Result<(), Box<dyn std::error::Error>> {
    let frame = state.build_frame();
    disable_raw_mode()?;
    print!("\x1B[2J\x1B[H{}", frame);
    enable_raw_mode()?;
    std::io::stdout().flush().unwrap();

    Ok(())
}
