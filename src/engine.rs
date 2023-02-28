use std::str::FromStr;
use std::sync::mpsc::Receiver;

use chess::{Board, ChessMove, MoveGen};
use rand::seq::SliceRandom;

use crate::engine_command::EngineCommand;
use crate::search_options::SearchOptions;

pub struct Engine {
    best_move: String,
    receiver: Receiver<EngineCommand>,
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Engine {
        Engine {
            best_move: String::from("bestmove None"),
            receiver,
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = self.receiver.recv().unwrap();

            if command.quit {
                break;
            } else if command.stop {
                continue;
            }

            self.search(command.search_options);
        }
    }

    fn search(&mut self, search_options: SearchOptions) {
        println!("info {:?}", search_options);
        if self.check_stop() {
            return;
        }

        let board = Engine::get_current_board(search_options.fen, search_options.played_moves);
        let movegen = MoveGen::new_legal(&board);
        let moves: Vec<_> = movegen.collect();

        let chosen_move = moves.choose(&mut rand::thread_rng()).unwrap();
        self.best_move = chosen_move.to_string();
        println!("bestmove {}", self.best_move);
    }

    fn check_stop(&self) -> bool {
        self.receiver
            .try_recv()
            .unwrap_or(EngineCommand::default())
            .stop
    }

    fn get_current_board(fen: String, played_moves: Vec<String>) -> Board {
        let mut board: Board =
            Board::from_str(fen.as_str()).expect("Board could not be created from fen.");
        for mv in played_moves {
            board = board
                .make_move_new(ChessMove::from_str(mv.as_str()).expect("Invalid move string."));
        }
        board
    }
}
