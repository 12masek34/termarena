use crate::game::state::Player;

#[derive(Debug)]
pub struct ClientState {
    pub player: Option<Player>,
}

impl ClientState {
    pub fn new() -> Self {
        Self { player: None }
    }

    pub fn init_player(&mut self, player: Player) {
        self.player = Some(player);
    }
}
