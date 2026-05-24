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
    pub infinite: bool,
    pub ponder: bool,

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
            infinite: false,
            ponder: false,

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
        if args.is_empty() {
            println!("info string Invalid position command.");
            return;
        }

        let mut board = Board::default();

        if args[0] == "fen" {
            if args.len() < 2 {
                println!("info string Invalid FEN.");
                return;
            }
            let mut fen = args[1].to_string();
            for partial in args[2..].as_ref() {
                if partial == "moves" {
                    break;
                }
                fen += &*String::from(" ");
                fen += partial;
            }
            board = match Board::from_str(fen.as_str()) {
                Ok(board) => board,
                Err(_) => {
                    println!("info string Invalid FEN.");
                    return;
                }
            };
        }

        let moves_start_index = args
            .iter()
            .position(|r| r == "moves")
            .unwrap_or(args.len() - 1)
            + 1;
        let played_moves = args[moves_start_index..].to_vec();

        let mut game = Game::new_with_board(board);
        for chess_move in played_moves {
            let parsed_move = match ChessMove::from_str(chess_move.as_str()) {
                Ok(parsed_move) => parsed_move,
                Err(_) => {
                    println!("info string Invalid move: {}", chess_move);
                    return;
                }
            };
            if !game.make_move(parsed_move) {
                println!("info string Illegal move: {}", chess_move);
                return;
            }
        }

        self.chess_game = game;
    }

    pub fn set_search_parameters(&mut self, args: &[String]) {
        self.reset_temporary_parameters();

        self.ponder = args.iter().any(|r| r == "ponder");
        let infinite_index = args.iter().position(|r| r == "infinite");
        if infinite_index.is_some() {
            self.depth = f64::INFINITY;
            self.infinite = true;
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
            self.move_time = Self::parse_usize(args, move_time_index.unwrap(), "movetime");
        }

        if white_time_index.is_some() {
            self.white_time = Self::parse_usize(args, white_time_index.unwrap(), "wtime");
        }
        if white_increment_index.is_some() {
            self.white_increment = Self::parse_usize(args, white_increment_index.unwrap(), "winc");
        }
        if black_time_index.is_some() {
            self.black_time = Self::parse_usize(args, black_time_index.unwrap(), "btime");
        }
        if black_increment_index.is_some() {
            self.black_increment = Self::parse_usize(args, black_increment_index.unwrap(), "binc");
        }
        if depth_index.is_some() {
            self.depth = Self::parse_f64(args, depth_index.unwrap(), "depth");
        }
    }

    pub fn set_option(&mut self, args: &[String]) {
        let name_index = args.iter().position(|r| r == "name");
        let value_index = args.iter().position(|r| r == "value");

        if !name_index.is_some()
            || !value_index.is_some()
            || name_index.unwrap() >= value_index.unwrap()
        {
            println!("Invalid setoption command.");
            return;
        }

        let option_name: &str = &args[name_index.unwrap() + 1..value_index.unwrap()]
            .join(" ")
            .to_lowercase();
        let value = &args[value_index.unwrap() + 1..].join(" ").to_lowercase();

        match option_name {
            "move overhead" => {
                if let Ok(move_overhead) = value.parse::<f64>()
                    && move_overhead.is_finite()
                    && (0.0..=5000.0).contains(&move_overhead)
                {
                    self.move_overhead = move_overhead;
                } else {
                    println!("info string Invalid Move Overhead value.");
                }
            }
            "threads" => {
                if let Ok(threads) = value.parse::<usize>() {
                    self.threads = threads.clamp(1, 1);
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

    fn parse_f64(args: &[String], index: usize, name: &str) -> f64 {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                2.
            }
        }
    }

    fn reset_temporary_parameters(&mut self) {
        self.move_time = 0;
        self.white_time = 0;
        self.white_increment = 0;
        self.black_time = 0;
        self.black_increment = 0;
        self.depth = f64::INFINITY;
        self.infinite = false;
        self.ponder = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    #[test]
    fn go_parameters_parse_and_reset_ponder_and_infinite_flags() {
        let mut options = SearchOptions::default();

        options.set_search_parameters(&args(&["ponder", "wtime", "1000", "btime", "1000"]));
        assert!(options.ponder);
        assert!(!options.infinite);
        assert_eq!(options.white_time, 1000);
        assert_eq!(options.black_time, 1000);

        options.set_search_parameters(&args(&["infinite"]));
        assert!(options.infinite);
        assert!(!options.ponder);

        options.set_search_parameters(&args(&["depth", "3"]));
        assert!(!options.infinite);
        assert!(!options.ponder);
        assert_eq!(options.depth, 3.0);
    }

    #[test]
    fn supplied_illegal_game_final_positions_parse() {
        let mut options = SearchOptions::default();

        options.set_position(&args(&[
            "fen",
            "8/6K1/8/8/8/p7/P7/1k6",
            "b",
            "-",
            "-",
            "4",
            "71",
        ]));
        assert_eq!(options.chess_game.side_to_move(), chess::Color::Black);

        options.set_position(&args(&[
            "fen",
            "8/8/8/K3R3/3Q4/8/6p1/2k4q",
            "w",
            "-",
            "-",
            "26",
            "91",
        ]));
        assert_eq!(options.chess_game.side_to_move(), chess::Color::White);
    }
}
