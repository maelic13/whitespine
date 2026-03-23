use std::path::PathBuf;
use std::str::FromStr;

use chess::{Board, ChessMove, Game};

use crate::heuristic_type::HeuristicType;

const DEFAULT_MODEL_FILENAME: &str = "v1_model2-100-2.onnx";

fn default_model_file() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    candidates.push(PathBuf::from(r"D:\code\beast\models\v1_model2-100-2.onnx"));
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("beast")
            .join("models")
            .join(DEFAULT_MODEL_FILENAME),
    );
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("models")
            .join(DEFAULT_MODEL_FILENAME),
    );

    candidates.into_iter().find(|path| path.exists())
}

fn max_threads() -> usize {
    std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub chess_game: Game,

    pub move_time: usize,
    pub white_time: usize,
    pub white_increment: usize,
    pub black_time: usize,
    pub black_increment: usize,
    pub depth: f64,

    pub fifty_moves_rule: bool,
    pub heuristic_type: HeuristicType,
    pub model_file: Option<PathBuf>,
    pub max_depth: f64,
    pub move_overhead: f64,
    pub syzygy_path: Option<PathBuf>,
    pub threads: usize,
}

impl SearchOptions {
    pub fn default() -> SearchOptions {
        let model_file = default_model_file();

        SearchOptions {
            chess_game: Game::new(),

            move_time: 0,
            white_time: 0,
            white_increment: 0,
            black_time: 0,
            black_increment: 0,
            depth: f64::INFINITY,

            fifty_moves_rule: true,
            heuristic_type: if model_file.is_some() {
                HeuristicType::NeuralNetwork
            } else {
                HeuristicType::Classical
            },
            model_file,
            max_depth: f64::INFINITY,
            move_overhead: 10.,
            syzygy_path: None,
            threads: 1,
        }
    }

    pub fn get_uci_options() -> Vec<String> {
        let defaults = SearchOptions::default();

        Vec::from([
            format!(
                "option name Heuristic type combo default {} var {} var {}",
                defaults.heuristic_type.as_str(),
                HeuristicType::Classical.as_str(),
                HeuristicType::NeuralNetwork.as_str()
            ),
            format!(
                "option name ModelFile type string default {}",
                defaults
                    .model_file
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_else(|| String::from("<empty>"))
            ),
            format!(
                "option name Threads type spin default {} min 1 max {}",
                defaults.threads,
                max_threads()
            ),
            String::from("option name MaxDepth type spin default -1 min -1 max 99"),
            String::from("option name Move Overhead type spin default 10 min 0 max 5000"),
            String::from("option name Syzygy50MoveRule type check default true"),
            String::from("option name SyzygyPath type string default <empty>"),
        ])
    }

    pub fn reset(&mut self) {
        self.chess_game = Game::new();
        self.reset_temporary_parameters();
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
        let raw_value = args[value_index.unwrap() + 1..].join(" ");
        let value = raw_value.trim();
        let value_lower = value.to_lowercase();

        match option_name {
            "maxdepth" => {
                let depth = value.parse::<f64>().unwrap();
                if depth == -1. {
                    self.max_depth = f64::INFINITY;
                } else {
                    self.max_depth = depth;
                }
            }
            "heuristic" | "heuristic type" => match HeuristicType::from_str(value) {
                Ok(heuristic_type) => self.heuristic_type = heuristic_type,
                Err(message) => println!("{}", message),
            },
            "modelfile" => {
                if value.is_empty() || value == "<empty>" {
                    self.model_file = None;
                    return;
                }

                let path = PathBuf::from(value);
                if !path.exists() {
                    println!("Invalid model file.");
                    return;
                }
                self.model_file = Some(path);
            }
            "threads" => {
                let threads = value.parse::<usize>().unwrap();
                if threads == 0 {
                    println!("Invalid thread limit.");
                    return;
                }
                self.threads = threads.min(max_threads());
            }
            "move overhead" => self.move_overhead = value.parse::<f64>().unwrap(),
            "syzygy50moverule" => self.fifty_moves_rule = value_lower == "true",
            "syzygypath" => {
                if value.is_empty() || value == "<empty>" {
                    self.syzygy_path = None;
                    return;
                }

                let path = PathBuf::from(value);
                if !path.exists() {
                    println!("Invalid syzygy path.");
                    return;
                }
                self.syzygy_path = Some(path);
            }
            _ => {}
        }
    }

    pub fn search_depth(&self) -> f64 {
        return [self.max_depth, self.depth]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
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
    use std::fs;

    use super::SearchOptions;
    use crate::heuristic_type::HeuristicType;

    #[test]
    fn set_option_updates_heuristic_type() {
        let mut options = SearchOptions::default();
        let args = vec![
            String::from("name"),
            String::from("Heuristic"),
            String::from("value"),
            String::from("classical"),
        ];

        options.set_option(&args);

        assert_eq!(options.heuristic_type, HeuristicType::Classical);
    }

    #[test]
    fn set_option_updates_model_file() {
        let mut options = SearchOptions::default();
        let temp_model = std::env::temp_dir().join("whitespine-test-model.onnx");
        fs::write(&temp_model, b"test").unwrap();

        let args = vec![
            String::from("name"),
            String::from("ModelFile"),
            String::from("value"),
            temp_model.to_string_lossy().to_string(),
        ];

        options.set_option(&args);

        assert_eq!(options.model_file, Some(temp_model.clone()));

        let _ = fs::remove_file(temp_model);
    }
}
