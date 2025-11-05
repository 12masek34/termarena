use super::ClientState;
use crossterm::{
    ExecutableCommand,
    cursor::{Hide, MoveTo, Show},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
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

pub fn render(state: &ClientState) {
    let mut frame = String::new();

    for (y, row) in state.map.tiles.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            if x == state.player_x && y == state.player_y {
                frame.push('@');
            } else {
                frame.push(*ch);
            }
        }
        frame.push('\n');
    }

    frame.push_str(&format!(
        "\nПозиция игрока: {} {}\n",
        state.player_x, state.player_y
    ));

    print!("\x1B[H");
    print!("{}", frame);
    std::io::stdout().flush().unwrap();
}
