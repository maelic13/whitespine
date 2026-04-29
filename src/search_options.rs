use std::str::FromStr;

use chess::{Board, ChessMove, Game};

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub chess_game: Game,

    pub move_time: usize,
    pub white_time: usize,
    pub white_increment: usize,
    pub black_time: usize,
    pub black_increment: usize,
    pub depth: f64,

    pub move_overhead: f64,
    pub threads: usize,
}

impl SearchOptions {
    pub fn default() -> SearchOptions {
        SearchOptions {
            chess_game: Game::new(),

            move_time: 0,
            white_time: 0,
            white_increment: 0,
            black_time: 0,
            black_increment: 0,
            depth: f64::INFINITY,

            move_overhead: 10.,
            threads: 1,
        }
    }

    pub fn get_uci_options() -> Vec<String> {
        Vec::from([
            String::from("option name Move Overhead type spin default 10 min 0 max 5000"),
            String::from("option name Threads type spin default 1 min 1 max 1"),
        ])
    }

    pub fn reset(&mut self) {
        self.chess_game = Game::new();
        self.reset_temporary_parameters();
    }

    pub fn set_position(&mut self, args: &[String]) {
        let mut board = Board::default();

        if args[0] == "fen" {
            let mut fen = args[1].to_string();
            for partial in args[2..].as_ref() {
                if partial == "moves" {
                    break;
                }
                fen += &*String::from(" ");
                fen += partial;
            }
            board = Board::from_str(fen.as_str()).expect("Board could not be created from fen.");
        }

        let moves_start_index = args
            .iter()
            .position(|r| r == "moves")
            .unwrap_or(args.len() - 1)
            + 1;
        let played_moves = args[moves_start_index..].to_vec();

        let mut game = Game::new_with_board(board);
        for chess_move in played_moves {
            game.make_move(ChessMove::from_str(chess_move.as_str()).expect("Invalid move string."));
        }

        self.chess_game = game;
    }

    pub fn set_search_parameters(&mut self, args: &[String]) {
        self.reset_temporary_parameters();

        let infinite_index = args.iter().position(|r| r == "infinite");
        if infinite_index.is_some() {
            self.depth = f64::INFINITY;
            return;
        }

        if args.is_empty() {
            self.depth = 2.;
        }

        let move_time_index = args.iter().position(|r| r == "movetime");
        let white_time_index = args.iter().position(|r| r == "wtime");
        let white_increment_index = args.iter().position(|r| r == "winc");
        let black_time_index = args.iter().position(|r| r == "btime");
        let black_increment_index = args.iter().position(|r| r == "binc");
        let depth_index = args.iter().position(|r| r == "depth");

        if move_time_index.is_some() {
            self.move_time = args[move_time_index.unwrap() + 1].parse().unwrap();
        }

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

    pub fn set_option(&mut self, args: &[String]) {
        let name_index = args.iter().position(|r| r == "name");
        let value_index = args.iter().position(|r| r == "value");

        if !name_index.is_some() || !value_index.is_some() {
            println!("Invalid setoption command.");
            return;
        }

        let option_name: &str = &args[name_index.unwrap() + 1..value_index.unwrap()]
            .join(" ")
            .to_lowercase();
        let value = &args[value_index.unwrap() + 1..].join(" ").to_lowercase();

        match option_name {
            "move overhead" => self.move_overhead = value.parse::<f64>().unwrap(),
            "threads" => {
                let threads = value.parse::<usize>().unwrap();
                self.threads = threads.clamp(1, 1);
            }
            _ => {}
        }
    }

    fn reset_temporary_parameters(&mut self) {
        self.move_time = 0;
        self.white_time = 0;
        self.white_increment = 0;
        self.black_time = 0;
        self.black_increment = 0;
        self.depth = f64::INFINITY;
    }
}
