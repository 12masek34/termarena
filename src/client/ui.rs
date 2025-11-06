use super::ClientState;
use crossterm::{
    ExecutableCommand,
    cursor::{Hide, Show},
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
    let map = match &state.map {
        Some(m) => m,
        None => return,
    };

    let mut frame = String::new();

    for (y, row) in map.tiles.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            let mut ch_to_draw = *ch;
            for (_, player) in state.players.iter() {
                if player.x == x && player.y == y {
                    ch_to_draw = '@';
                    break;
                }
            }
            frame.push(ch_to_draw);
        }
        frame.push('\n');
    }

    frame.push_str(&format!(
        "\nID: {}\n",
        state.id.map_or("unknown".to_string(), |id| id.to_string())
    ));

    print!("\x1B[H");
    print!("{}", frame);
    std::io::stdout().flush().unwrap();
}
