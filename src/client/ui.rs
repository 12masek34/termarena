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
    let map = match &state.map {
        Some(m) => m,
        None => return Ok(()),
    };

    let mut frame_rows: Vec<String> = vec![];

    for (y, row) in map.tiles.iter().enumerate() {
        let mut row_str = row.iter().collect::<String>();

        for (_id, player) in state.players.iter() {
            if player.y == y && player.x < row_str.len() {
                row_str.replace_range(player.x..player.x + 1, "@");
            }
        }

        frame_rows.push(row_str);
    }

    let mut frame = frame_rows.join("\n");

    let (player_x, player_y) = if let Some(id) = state.id {
        if let Some(player) = state.players.get(&id) {
            (player.x, player.y)
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    frame.push_str(&format!(
        "\nYou: {}\nPosition: ({},{})\nPlayers nearby: {}\nMap size: {}x{}\n",
        state.id.unwrap_or(0),
        player_x,
        player_y,
        state.players.len(),
        map.width,
        map.height,
    ));

    disable_raw_mode()?;
    print!("\x1B[2J\x1B[H{}", frame);
    enable_raw_mode()?;
    std::io::stdout().flush().unwrap();

    Ok(())
}
