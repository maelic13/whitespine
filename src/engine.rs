use std::sync::mpsc::Receiver;

use chess::{Board, BoardStatus, ChessMove, Game, MoveGen};
use rand::seq::SliceRandom;
use stopwatch::Stopwatch;

use crate::engine_command::EngineCommand;
use crate::heuristic::Heuristic;
use crate::search_options::SearchOptions;

pub struct Engine {
    heuristic: Heuristic,
    receiver: Receiver<EngineCommand>,
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Engine {
        Engine {
            heuristic: Heuristic::default(),
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

            self.initialize_heuristic(&command.search_options);
            self.search(command.search_options.board, command.search_options.depth);
        }
    }

    fn initialize_heuristic(&mut self, _search_options: &SearchOptions) {
        self.heuristic.fifty_moves_rule = true;
        self.heuristic.syzygy_path = None;
    }

    fn search(&mut self, board: Board, max_depth: f64) {
        // start with random move choice, to be used in case of timeout before first depth is reached
        let move_gen = MoveGen::new_legal(&board);
        let possible_moves: Vec<_> = move_gen.collect();
        let mut moves: Vec<ChessMove> = vec![possible_moves
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_owned()];

        let mut stop_watch = Stopwatch::new();
        stop_watch.start();
        let mut depth: f64 = 0.;
        let mut evaluation: f64;
        let game = Game::new_with_board(board);
        let mut nodes_searched: usize = 0;

        while depth < max_depth {
            depth += 1.;

            let result = self.negamax(game.clone(), depth, f64::NEG_INFINITY, f64::INFINITY);
            match result {
                Ok((eval, pv, nodes)) => {
                    evaluation = eval;
                    nodes_searched += nodes;
                    moves = pv;
                }
                Err(_) => {
                    break;
                }
            }

            let mut string_moves: Vec<String> = vec![];
            for chess_move in &moves {
                string_moves.push(chess_move.to_string());
            }

            println!(
                "info depth {} score cp {} nodes {} nps {} time {} pv {}",
                depth,
                evaluation as usize,
                nodes_searched,
                (nodes_searched as f64 / stop_watch.elapsed().as_secs() as f64) as usize,
                stop_watch.elapsed().as_secs(),
                string_moves.join(" ")
            )
        }

        println!("bestmove {}", &moves[0].to_string());
    }

    fn negamax(&self, game: Game, depth: f64, mut alpha: f64, beta: f64
    ) -> Result<(f64, Vec<ChessMove>, usize), &'static str> {
        if self.check_stop() {
            return Err("Calculation stopped.");
        }

        let mut nodes_searched: usize = 1;

        if game.current_position().status() != BoardStatus::Ongoing || depth == 0. {
            return Ok((
                self.heuristic.clone().evaluate(game),
                vec![],
                nodes_searched,
            ));
        }

        let move_gen = MoveGen::new_legal(&game.current_position());
        let mut best_moves: Vec<ChessMove> = vec![];
        let mut moves: Vec<ChessMove>;
        let mut current_game: Game;
        let mut evaluation: f64;

        for chess_move in move_gen {
            current_game = game.clone();
            current_game.make_move(chess_move);

            let result = self.negamax(current_game, depth - 1., -beta, -alpha);
            match result {
                Ok((eval, pv, nodes)) => {
                    evaluation = eval;
                    nodes_searched += nodes;
                    moves = pv;
                }
                Err(message) => return Err(message),
            }

            evaluation *= -1.;
            moves.insert(0, chess_move);

            if evaluation >= beta {
                return Ok((beta, vec![], nodes_searched));
            }
            if evaluation > alpha {
                alpha = evaluation;
                best_moves = moves;
            }
        }

        Ok((alpha, best_moves, nodes_searched))
    }

    fn check_stop(&self) -> bool {
        let command = self.receiver.try_recv().unwrap_or(EngineCommand::default());
        command.stop || command.quit
    }
}
