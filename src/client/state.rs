#[derive(Debug)]
pub struct ClientState {
    pub id: Option<u32>,
    pub players: HashMap<u32, Player>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            id: None,
            players: HashMap::new(),
        }
    }
}
