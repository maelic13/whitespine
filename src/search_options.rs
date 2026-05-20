use crate::board::board::Board;
use crate::board::movegen::generate_legal_moves;
use crate::board::moves::Move;

const MAX_HASH_MB: usize = 33_554_432;

#[derive(Clone)]
pub struct SearchOptions {
    pub board: Board,
    pub move_time: u64,
    pub white_time: u64,
    pub white_increment: u64,
    pub black_time: u64,
    pub black_increment: u64,
    pub depth: u32,
    pub move_overhead: u64,
    pub threads: usize,
    pub hash_mb: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            board: Board::starting_position(),
            move_time: 0,
            white_time: 0,
            white_increment: 0,
            black_time: 0,
            black_increment: 0,
            depth: u32::MAX,
            move_overhead: 10,
            threads: 1,
            hash_mb: 64,
        }
    }
}

impl SearchOptions {
    pub fn get_uci_options() -> Vec<String> {
        vec![
            String::from("option name Move Overhead type spin default 10 min 0 max 5000"),
            String::from("option name Threads type spin default 1 min 1 max 1"),
            format!(
                "option name Hash type spin default 64 min 1 max {}",
                MAX_HASH_MB
            ),
        ]
    }

    pub fn reset(&mut self) {
        self.board = Board::starting_position();
        self.reset_temporary_parameters();
    }

    pub fn set_position(&mut self, args: &[String]) {
        if args.is_empty() {
            println!("info string Invalid position command.");
            return;
        }

        let mut index;
        let mut board = if args[0] == "startpos" {
            index = 1;
            Board::starting_position()
        } else if args[0] == "fen" {
            index = 1;
            let mut fen_parts = Vec::new();
            while index < args.len() && args[index] != "moves" {
                fen_parts.push(args[index].as_str());
                index += 1;
            }
            if fen_parts.is_empty() {
                println!("info string Invalid FEN.");
                return;
            }
            match Board::from_fen(&fen_parts.join(" ")) {
                Ok(board) => board,
                Err(_) => {
                    println!("info string Invalid FEN.");
                    return;
                }
            }
        } else {
            println!("info string Invalid position command.");
            return;
        };

        if index < args.len() && args[index] == "moves" {
            index += 1;
            while index < args.len() {
                let mv_str = &args[index];
                match parse_uci_move(&board, mv_str) {
                    Some(mv) => board.make_move(mv),
                    None => {
                        println!("info string Illegal move: {}", mv_str);
                        return;
                    }
                }
                index += 1;
            }
        }

        self.board = board;
    }

    pub fn set_search_parameters(&mut self, args: &[String]) {
        self.reset_temporary_parameters();

        if args.iter().any(|arg| arg == "infinite") {
            self.depth = u32::MAX;
            return;
        }

        if args.is_empty() {
            self.depth = 2;
            return;
        }

        if let Some(index) = args.iter().position(|arg| arg == "movetime") {
            self.move_time = Self::parse_u64(args, index, "movetime");
        }
        if let Some(index) = args.iter().position(|arg| arg == "wtime") {
            self.white_time = Self::parse_u64(args, index, "wtime");
        }
        if let Some(index) = args.iter().position(|arg| arg == "winc") {
            self.white_increment = Self::parse_u64(args, index, "winc");
        }
        if let Some(index) = args.iter().position(|arg| arg == "btime") {
            self.black_time = Self::parse_u64(args, index, "btime");
        }
        if let Some(index) = args.iter().position(|arg| arg == "binc") {
            self.black_increment = Self::parse_u64(args, index, "binc");
        }
        if let Some(index) = args.iter().position(|arg| arg == "depth") {
            self.depth = Self::parse_u32(args, index, "depth").max(1);
        }
    }

    pub fn set_option(&mut self, args: &[String]) {
        let name_index = args.iter().position(|arg| arg == "name");
        let value_index = args.iter().position(|arg| arg == "value");

        if name_index.is_none()
            || value_index.is_none()
            || name_index.unwrap() >= value_index.unwrap()
        {
            println!("info string Invalid setoption command.");
            return;
        }

        let option_name = args[name_index.unwrap() + 1..value_index.unwrap()]
            .join(" ")
            .to_lowercase();
        let value = args[value_index.unwrap() + 1..].join(" ").to_lowercase();

        match option_name.as_str() {
            "move overhead" => match value.parse::<u64>() {
                Ok(parsed) if parsed <= 5000 => self.move_overhead = parsed,
                _ => println!("info string Invalid Move Overhead value."),
            },
            "threads" => match value.parse::<usize>() {
                Ok(parsed) => self.threads = parsed.clamp(1, 1),
                Err(_) => println!("info string Invalid Threads value."),
            },
            "hash" => match value.parse::<usize>() {
                Ok(parsed) => self.hash_mb = parsed.clamp(1, MAX_HASH_MB),
                Err(_) => println!("info string Invalid Hash value."),
            },
            _ => {}
        }
    }

    fn parse_u64(args: &[String], index: usize, name: &str) -> u64 {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                0
            }
        }
    }

    fn parse_u32(args: &[String], index: usize, name: &str) -> u32 {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                2
            }
        }
    }

    fn reset_temporary_parameters(&mut self) {
        self.move_time = 0;
        self.white_time = 0;
        self.white_increment = 0;
        self.black_time = 0;
        self.black_increment = 0;
        self.depth = u32::MAX;
    }
}

fn parse_uci_move(board: &Board, text: &str) -> Option<Move> {
    generate_legal_moves(board)
        .into_iter()
        .find(|mv| mv.to_string() == text)
}
