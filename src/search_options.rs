use std::path::PathBuf;
use std::str::FromStr;

use chess::{Board, ChessMove, Game, Piece};

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub chess_game: Game,
    pub current_board: Board,
    pub position_hash_history: Vec<u64>,
    pub reversible_moves: usize,

    pub move_time: usize,
    pub white_time: usize,
    pub white_increment: usize,
    pub black_time: usize,
    pub black_increment: usize,
    pub depth: f64,

    pub fifty_moves_rule: bool,
    pub max_depth: f64,
    pub move_overhead: f64,
    pub syzygy_path: Option<PathBuf>,
}

impl SearchOptions {
    pub fn default() -> SearchOptions {
        let board = Board::default();

        SearchOptions {
            chess_game: Game::new(),
            current_board: board,
            position_hash_history: vec![board.get_hash()],
            reversible_moves: 0,

            move_time: 0,
            white_time: 0,
            white_increment: 0,
            black_time: 0,
            black_increment: 0,
            depth: f64::INFINITY,

            fifty_moves_rule: true,
            max_depth: f64::INFINITY,
            move_overhead: 10.,
            syzygy_path: None,
        }
    }

    pub fn get_uci_options() -> Vec<String> {
        Vec::from([
            String::from("option name MaxDepth type spin default -1 min -1 max 99"),
            String::from("option name Move Overhead type spin default 10 min 0 max 5000"),
            String::from("option name Syzygy50MoveRule type check default true"),
            String::from("option name SyzygyPath type string default <empty>"),
        ])
    }

    pub fn reset(&mut self) {
        *self = SearchOptions::default();
    }

    pub fn set_position(&mut self, args: &[String]) {
        if args.is_empty() {
            return;
        }

        let mut board = Board::default();

        if args[0] == "fen" {
            if args.len() < 2 {
                return;
            }

            let mut fen = args[1].to_string();
            for partial in &args[2..] {
                if partial == "moves" {
                    break;
                }
                fen.push(' ');
                fen.push_str(partial);
            }

            board = Board::from_str(&fen).expect("Board could not be created from fen.");
        }

        let played_moves = args
            .iter()
            .position(|arg| arg == "moves")
            .map(|index| {
                args[index + 1..]
                    .iter()
                    .map(|mv| ChessMove::from_str(mv).expect("Invalid move string."))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut game = Game::new_with_board(board);
        for chess_move in &played_moves {
            game.make_move(*chess_move);
        }

        let (current_board, position_hash_history, reversible_moves) =
            SearchOptions::build_position_state(board, &played_moves);

        self.chess_game = game;
        self.current_board = current_board;
        self.position_hash_history = position_hash_history;
        self.reversible_moves = reversible_moves;
    }

    pub fn set_search_parameters(&mut self, args: &[String]) {
        self.reset_temporary_parameters();

        if args.iter().any(|arg| arg == "infinite") {
            self.depth = f64::INFINITY;
            return;
        }

        if args.is_empty() {
            self.depth = 2.;
        }

        let move_time_index = args.iter().position(|arg| arg == "movetime");
        let white_time_index = args.iter().position(|arg| arg == "wtime");
        let white_increment_index = args.iter().position(|arg| arg == "winc");
        let black_time_index = args.iter().position(|arg| arg == "btime");
        let black_increment_index = args.iter().position(|arg| arg == "binc");
        let depth_index = args.iter().position(|arg| arg == "depth");

        if let Some(index) = move_time_index {
            self.move_time = args[index + 1].parse().unwrap();
        }
        if let Some(index) = white_time_index {
            self.white_time = args[index + 1].parse().unwrap();
        }
        if let Some(index) = white_increment_index {
            self.white_increment = args[index + 1].parse().unwrap();
        }
        if let Some(index) = black_time_index {
            self.black_time = args[index + 1].parse().unwrap();
        }
        if let Some(index) = black_increment_index {
            self.black_increment = args[index + 1].parse().unwrap();
        }
        if let Some(index) = depth_index {
            self.depth = args[index + 1].parse().unwrap();
        }
    }

    pub fn set_option(&mut self, args: &[String]) {
        let name_index = args.iter().position(|arg| arg == "name");
        let value_index = args.iter().position(|arg| arg == "value");

        if name_index.is_none() || value_index.is_none() {
            println!("Invalid setoption command.");
            return;
        }

        let name_index = name_index.unwrap();
        let value_index = value_index.unwrap();
        let option_name = args[name_index + 1..value_index].join(" ").to_lowercase();
        let value = args[value_index + 1..].join(" ").to_lowercase();

        match option_name.as_str() {
            "maxdepth" => {
                let depth = value.parse::<f64>().unwrap();
                self.max_depth = if depth == -1. { f64::INFINITY } else { depth };
            }
            "move overhead" => self.move_overhead = value.parse::<f64>().unwrap(),
            "syzygy50moverule" => self.fifty_moves_rule = value == "true",
            "syzygypath" => {
                let path = PathBuf::from(value);
                self.syzygy_path = if path.exists() { Some(path) } else { None };
            }
            _ => {}
        }
    }

    pub fn search_depth(&self) -> f64 {
        [self.max_depth, self.depth]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b))
    }

    fn build_position_state(
        start_board: Board,
        played_moves: &[ChessMove],
    ) -> (Board, Vec<u64>, usize) {
        let mut board = start_board;
        let mut reversible_moves = 0;
        let mut position_hash_history = vec![board.get_hash()];

        for chess_move in played_moves {
            let previous_white_rights = board.castle_rights(chess::Color::White);
            let previous_black_rights = board.castle_rights(chess::Color::Black);
            let moving_piece = board.piece_on(chess_move.get_source());
            let is_en_passant = moving_piece == Some(Piece::Pawn)
                && chess_move.get_source().get_file() != chess_move.get_dest().get_file()
                && board.piece_on(chess_move.get_dest()).is_none();
            let is_capture = board.piece_on(chess_move.get_dest()).is_some() || is_en_passant;

            board = board.make_move_new(*chess_move);

            let castle_rights_changed = board.castle_rights(chess::Color::White)
                != previous_white_rights
                || board.castle_rights(chess::Color::Black) != previous_black_rights;
            let irreversible =
                moving_piece == Some(Piece::Pawn) || is_capture || castle_rights_changed;

            if irreversible {
                reversible_moves = 0;
                position_hash_history.clear();
            } else {
                reversible_moves += 1;
            }

            position_hash_history.push(board.get_hash());
        }

        (board, position_hash_history, reversible_moves)
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

#[cfg(test)]
mod tests {
    use super::SearchOptions;
    use chess::Board;

    #[test]
    fn set_position_handles_empty_args() {
        let mut options = SearchOptions::default();
        options.set_position(&[]);
        assert_eq!(
            options.current_board.to_string(),
            Board::default().to_string()
        );
    }

    #[test]
    fn set_position_supports_startpos_without_moves() {
        let mut options = SearchOptions::default();
        let args = vec!["startpos".to_string()];
        options.set_position(&args);
        assert_eq!(
            options.current_board.to_string(),
            Board::default().to_string()
        );
    }

    #[test]
    fn set_position_handles_incomplete_fen_command() {
        let mut options = SearchOptions::default();
        let args = vec!["fen".to_string()];
        options.set_position(&args);
        assert_eq!(
            options.current_board.to_string(),
            Board::default().to_string()
        );
    }

    #[test]
    fn reversible_state_resets_after_irreversible_move() {
        let mut options = SearchOptions::default();
        let args = vec![
            "startpos".to_string(),
            "moves".to_string(),
            "g1f3".to_string(),
            "g8f6".to_string(),
            "e2e4".to_string(),
        ];
        options.set_position(&args);

        assert_eq!(options.reversible_moves, 0);
        assert_eq!(options.position_hash_history.len(), 1);
        assert_eq!(
            *options.position_hash_history.last().unwrap(),
            options.current_board.get_hash()
        );
    }
}
