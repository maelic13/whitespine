use crate::board::{Board, Move};

pub const MAX_THREADS: usize = 1024;

#[derive(Copy, Clone)]
pub struct EngineOptions {
    pub move_overhead: f64,
    pub hash_mb: usize,
    pub clear_hash: bool,
    pub threads: usize,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            move_overhead: 10.0,
            hash_mb: 64,
            clear_hash: false,
            threads: 1,
        }
    }
}

#[derive(Clone)]
pub struct PositionState {
    pub board: Board,
}

impl Default for PositionState {
    fn default() -> Self {
        Self {
            board: Board::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SearchLimits {
    pub move_time: usize,
    pub white_time: usize,
    pub white_increment: usize,
    pub black_time: usize,
    pub black_increment: usize,
    pub depth: f64,
    pub movestogo: usize,
    pub nodes: u64,
    pub ponder: bool,
}

impl SearchLimits {
    fn reset_temporary_parameters(&mut self) {
        self.move_time = 0;
        self.white_time = 0;
        self.white_increment = 0;
        self.black_time = 0;
        self.black_increment = 0;
        self.depth = f64::INFINITY;
        self.movestogo = 0;
        self.nodes = 0;
        self.ponder = false;
    }
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            move_time: 0,
            white_time: 0,
            white_increment: 0,
            black_time: 0,
            black_increment: 0,
            depth: f64::INFINITY,
            movestogo: 0,
            nodes: 0,
            ponder: false,
        }
    }
}

#[derive(Clone, Default)]
pub struct SearchOptions {
    pub position: PositionState,
    pub engine: EngineOptions,
    pub limits: SearchLimits,
}

impl SearchOptions {
    pub fn get_uci_options() -> Vec<String> {
        Vec::from([
            String::from("option name Hash type spin default 64 min 1 max 33554432"),
            String::from("option name Clear Hash type button"),
            String::from("option name Move Overhead type spin default 10 min 0 max 5000"),
            format!("option name Threads type spin default 1 min 1 max {MAX_THREADS}"),
        ])
    }

    pub fn reset(&mut self) {
        self.position = PositionState::default();
        self.limits.reset_temporary_parameters();
    }

    pub fn set_position(&mut self, args: &[String]) {
        if args.is_empty() {
            println!("info string Invalid position command.");
            return;
        }

        let mut board = if args[0] == "startpos" {
            Board::default()
        } else if args[0] == "fen" {
            let fen_parts: Vec<&str> = args[1..]
                .iter()
                .take_while(|part| part.as_str() != "moves")
                .map(String::as_str)
                .collect();
            if fen_parts.is_empty() {
                println!("info string Invalid FEN.");
                return;
            }
            let fen = fen_parts.join(" ");
            match Board::from_fen(&fen) {
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

        let moves_start_index = args
            .iter()
            .position(|part| part == "moves")
            .map_or(args.len(), |index| index + 1);

        for move_text in &args[moves_start_index..] {
            if Move::from_uci(move_text).is_none() {
                println!("info string Invalid move: {}", move_text);
                return;
            }
            if !board.play_uci(move_text) {
                println!("info string Illegal move: {}", move_text);
                return;
            }
        }

        self.position.board = board;
    }

    pub fn set_search_parameters(&mut self, args: &[String]) {
        self.limits.reset_temporary_parameters();

        let infinite_index = args.iter().position(|r| r == "infinite");
        if infinite_index.is_some() {
            self.limits.depth = f64::INFINITY;
            return;
        }

        if args.is_empty() {
            self.limits.depth = 2.0;
        }

        let move_time_index = args.iter().position(|r| r == "movetime");
        let white_time_index = args.iter().position(|r| r == "wtime");
        let white_increment_index = args.iter().position(|r| r == "winc");
        let black_time_index = args.iter().position(|r| r == "btime");
        let black_increment_index = args.iter().position(|r| r == "binc");
        let depth_index = args.iter().position(|r| r == "depth");
        let movestogo_index = args.iter().position(|r| r == "movestogo");
        let nodes_index = args.iter().position(|r| r == "nodes");

        self.limits.ponder = args.iter().any(|r| r == "ponder");

        if let Some(index) = move_time_index {
            self.limits.move_time = Self::parse_usize(args, index, "movetime");
        }

        if let Some(index) = white_time_index {
            self.limits.white_time = Self::parse_usize(args, index, "wtime");
        }
        if let Some(index) = white_increment_index {
            self.limits.white_increment = Self::parse_usize(args, index, "winc");
        }
        if let Some(index) = black_time_index {
            self.limits.black_time = Self::parse_usize(args, index, "btime");
        }
        if let Some(index) = black_increment_index {
            self.limits.black_increment = Self::parse_usize(args, index, "binc");
        }
        if let Some(index) = depth_index {
            self.limits.depth = Self::parse_f64(args, index, "depth");
        }
        if let Some(index) = movestogo_index {
            self.limits.movestogo = Self::parse_usize(args, index, "movestogo");
        }
        if let Some(index) = nodes_index {
            self.limits.nodes = Self::parse_u64(args, index, "nodes");
        }
    }

    pub fn set_option(&mut self, args: &[String]) {
        let name_index = args.iter().position(|r| r == "name");
        let value_index = args.iter().position(|r| r == "value");

        if name_index.is_none() {
            println!("Invalid setoption command.");
            return;
        }

        let name_end = value_index.unwrap_or(args.len());
        if name_index.unwrap() >= name_end {
            println!("Invalid setoption command.");
            return;
        }

        let option_name: &str = &args[name_index.unwrap() + 1..name_end]
            .join(" ")
            .to_lowercase();
        let value = value_index
            .map(|index| args[index + 1..].join(" ").to_lowercase())
            .unwrap_or_default();

        match option_name {
            "hash" => {
                if let Ok(hash_mb) = value.parse::<usize>() {
                    self.engine.hash_mb = hash_mb.clamp(1, 33_554_432);
                } else {
                    println!("info string Invalid Hash value.");
                }
            }
            "clear hash" => {
                self.engine.clear_hash = true;
            }
            "move overhead" => {
                if let Ok(move_overhead) = value.parse::<f64>()
                    && move_overhead.is_finite()
                    && (0.0..=5000.0).contains(&move_overhead)
                {
                    self.engine.move_overhead = move_overhead;
                } else {
                    println!("info string Invalid Move Overhead value.");
                }
            }
            "threads" => {
                if let Ok(threads) = value.parse::<usize>() {
                    self.engine.threads = threads.clamp(1, MAX_THREADS);
                } else {
                    println!("info string Invalid Threads value.");
                }
            }
            _ => {}
        }
    }

    fn parse_usize(args: &[String], index: usize, name: &str) -> usize {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                0
            }
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

    fn parse_f64(args: &[String], index: usize, name: &str) -> f64 {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                2.0
            }
        }
    }
}
