const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const INFINITE_DEPTH: usize = 1000;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub fen: String,
    pub played_moves: Vec<String>,
    pub white_time: usize,
    pub white_increment: usize,
    pub black_time: usize,
    pub black_increment: usize,
    pub depth: usize,
}

impl SearchOptions {
    pub fn default() -> SearchOptions {
        SearchOptions {
            fen: String::from(START_POSITION),
            played_moves: vec![],
            white_time: 0,
            white_increment: 0,
            black_time: 0,
            black_increment: 0,
            depth: 2,
        }
    }

    pub fn reset(&mut self) {
        self.reset_position();
        self.reset_search_parameters();
    }

    pub fn set_position(&mut self, args: &[String]) {
        if args[0] == "fen" {
            let mut fen = args[1].to_string();
            for partial in args[2..].as_ref() {
                if partial == "moves" {
                    break;
                }
                fen += &*String::from(" ");
                fen += partial;
            }
            self.fen = fen;
        }

        let moves_start_index = args
            .iter()
            .position(|r| r == "moves")
            .unwrap_or(args.len() - 1)
            + 1;
        self.played_moves = args[moves_start_index..].to_vec();
    }

    pub fn set_search_parameters(&mut self, args: &[String]) {
        self.reset_search_parameters();

        let infinite_index = args.iter().position(|r| r == "infinite");
        if infinite_index.is_some() {
            self.depth = INFINITE_DEPTH;
            return;
        }

        let white_time_index = args.iter().position(|r| r == "wtime");
        let white_increment_index = args.iter().position(|r| r == "winc");
        let black_time_index = args.iter().position(|r| r == "btime");
        let black_increment_index = args.iter().position(|r| r == "binc");
        let depth_index = args.iter().position(|r| r == "depth");

        if white_time_index.is_some() {
            self.white_time = args[white_time_index.unwrap() + 1].parse().unwrap();
        }
        if white_increment_index.is_some() {
            self.white_increment = args[white_increment_index.unwrap() + 1].parse().unwrap();
        }
        if black_time_index.is_some() {
            self.black_time = args[black_time_index.unwrap() + 1].parse().unwrap();
        }
        if black_increment_index.is_some() {
            self.black_increment = args[black_increment_index.unwrap() + 1].parse().unwrap();
        }
        if depth_index.is_some() {
            self.depth = args[depth_index.unwrap() + 1].parse().unwrap();
        }
    }

    fn reset_position(&mut self) {
        self.fen = String::from(START_POSITION);
        self.played_moves = vec![];
    }

    fn reset_search_parameters(&mut self) {
        self.white_time = 0;
        self.white_increment = 0;
        self.black_time = 0;
        self.black_increment = 0;
        self.depth = 2;
    }
}
