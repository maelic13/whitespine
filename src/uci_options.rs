const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone)]
pub struct UciOptions {
    pub fen: String,
    pub played_moves: Vec<String>,
}

impl UciOptions {
    pub fn default() -> UciOptions {
        UciOptions {
            fen: String::from(START_POSITION),
            played_moves: vec![],
        }
    }

    pub fn reset_position(&mut self) {
        self.fen = String::from(START_POSITION);
        self.played_moves = vec![];
    }
}
