use super::ClientState;
use std::io::Write;

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
