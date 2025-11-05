use super::ClientState;

pub fn render(state: &ClientState) {
    let map = &state.map;

    for (y, row) in map.tiles.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            if x == state.player_x && y == state.player_y {
                print!("@");
            } else {
                print!("{}", ch);
            }
        }
        println!();
    }

    println!();
    println!("Позиция игрока: {} {}", state.player_x, state.player_y);
}
