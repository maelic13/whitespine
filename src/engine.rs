use std::sync::mpsc::{channel, Receiver};

use chess::{Board, BoardStatus, ChessMove, Color, Game, MoveGen, Piece, Square};
use chrono::Duration;
use rand::seq::SliceRandom;
use stopwatch::Stopwatch;
use timer::{Guard, Timer};

use crate::engine_command::EngineCommand;
use crate::heuristic::Heuristic;
use crate::piece_value::PieceValue;
use crate::search_options::SearchOptions;

pub struct Engine {
    heuristic: Heuristic,
    receiver: Receiver<EngineCommand>,
    timer: Timer,
    timer_guard: Option<Guard>,
    timer_receiver: Option<Receiver<bool>>,
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Engine {
        Engine {
            heuristic: Heuristic::default(),
            receiver,
            timer: Timer::new(),
            timer_guard: None,
            timer_receiver: None,
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
            self.start_timer(&command.search_options);
            let max_depth = [command.search_options.depth, command.search_options.max_depth]
            .iter().fold(f64::INFINITY, |a, &b| a.min(b));
            self.search(command.search_options.board, max_depth);
        }
    }

    fn initialize_heuristic(&mut self, search_options: &SearchOptions) {
        self.heuristic.fifty_moves_rule = search_options.fifty_moves_rule;
        self.heuristic.syzygy_path = search_options.syzygy_path.clone();
    }

    fn check_stop(&self) -> bool {
        let command = self.receiver.try_recv().unwrap_or(EngineCommand::default());
        let timeout;
        if self.timer_receiver.is_some() {
            timeout = self.timer_receiver.as_ref().unwrap().try_recv().unwrap_or(false);
        } else {
            timeout = false;
        }
        return command.stop || command.quit || timeout;
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

            let result = self.negamax(&game, depth, f64::NEG_INFINITY, f64::INFINITY);
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
                evaluation as isize,
                nodes_searched,
                (1_000_000. * nodes_searched as f64 / (stop_watch.elapsed().as_micros()) as f64) as usize,
                stop_watch.elapsed().as_millis(),
                string_moves.join(" ")
            )
        }

        println!("bestmove {}", &moves[0].to_string());
    }

    fn negamax(&self, game: &Game, depth: f64, mut alpha: f64, beta: f64
    ) -> Result<(f64, Vec<ChessMove>, usize), &'static str> {
        if self.check_stop() {
            return Err("Calculation stopped.");
        }

        let mut nodes_searched: usize = 1;

        if game.current_position().status() != BoardStatus::Ongoing {
            return Ok((self.heuristic.evaluate(game), vec![], nodes_searched));
        }
        if depth == 0. {
            let evaluation: f64;
            let result = self.quiescence(game, alpha, beta);
            match result {
                Ok((eval, nodes)) => {
                    evaluation = eval;
                    nodes_searched += nodes;
                }
                Err(message) => return Err(message),
            }
            return Ok((evaluation, vec![], nodes_searched))
        }

        let move_gen = MoveGen::new_legal(&game.current_position());
        let mut best_moves: Vec<ChessMove> = vec![];
        let mut moves: Vec<ChessMove>;
        let mut current_game: Game;
        let mut evaluation: f64;

        for chess_move in move_gen {
            current_game = game.clone();
            current_game.make_move(chess_move);

            let result = self.negamax(&current_game, depth - 1., -beta, -alpha);
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
    
    fn quiescence(&self, game: &Game, mut alpha: f64, beta: f64) -> Result<(f64, usize), &'static str> {
        if self.check_stop() {
            return Err("Calculation stopped.");
        }
        
        let evaluation = self.heuristic.evaluate(game);
        
        if evaluation >= beta { 
            return Ok((beta, 0));
        }
        
        let use_delta_pruning = game.current_position().combined().collect::<Vec<Square>>().len() == 8;
        if use_delta_pruning {
            if evaluation < alpha - 1000. {
                return Ok((alpha, 0));
            }
        }
        
        if evaluation > alpha { 
            alpha = evaluation;
        }

        let mut nodes_searched: usize = 0;
        let piece_value = PieceValue::default();
        for (chess_move, is_capture, is_en_passant) in Engine::get_captures_and_checks(&game.current_position()) {
            if use_delta_pruning && is_en_passant && (evaluation + piece_value.pawn_value < alpha) {
                continue 
            }
            else if use_delta_pruning && is_capture 
                && (piece_value.get_piece_value(game.current_position().piece_on(chess_move.get_dest()).unwrap()) 
                + piece_value.pawn_value < alpha) {
                continue
            }

            let mut current_game = game.clone();
            current_game.make_move(chess_move);
            nodes_searched += 1;

            let score: f64;
            let result = self.quiescence(&current_game, -beta, -alpha);
            match result {
                Ok((eval, nodes)) => {
                    score = -eval;
                    nodes_searched += nodes;
                }
                Err(message) => return Err(message),
            }

            if score >= beta {
                return Ok((beta, nodes_searched));
            }
            if score > alpha {
                alpha = score;
            }
        }

        return Ok((alpha, nodes_searched))
    }

    fn get_captures_and_checks(board: &Board) -> Vec<(ChessMove, bool, bool)> {
        let mut captures_and_checks: Vec<(ChessMove, bool, bool)> = vec![];
        let move_gen = MoveGen::new_legal(board);
        
        for chess_move in move_gen {
            let board_after_move = board.make_move_new(chess_move);
            
            let captured_piece = board.piece_on(chess_move.get_dest()) != None;
            let is_check = board_after_move.checkers().collect::<Vec<Square>>().len() != 0;
            
            let en_passant_capture = 
                board.piece_on(chess_move.get_source()).unwrap() == Piece::Pawn 
                    && (chess_move.get_source().get_rank() != chess_move.get_dest().get_rank())
                    && (chess_move.get_source().get_file() != chess_move.get_dest().get_file());
            
            if captured_piece || en_passant_capture || is_check {
                captures_and_checks.push((chess_move, captured_piece, en_passant_capture));
            }
        }
        
        return captures_and_checks;
    }

    fn start_timer(&mut self, search_options: &SearchOptions) {
        /* Start timer which will send timeout message. */
        if !search_options.has_time_options() {
            // do not start timer
            return;
        }

        let mut time_for_move: Option<f64> = None;
        let time_flex: f64 = 10.;

        if search_options.move_time != 0 {
            time_for_move = Some(search_options.move_time as f64 - time_flex);
        }
        if search_options.board.side_to_move() == Color::White && search_options.white_time != 0 {
            time_for_move = Some(0.2 * search_options.white_time as f64 - time_flex)
        }
        if search_options.board.side_to_move() == Color::Black && search_options.black_time != 0 {
            time_for_move = Some(0.2 * search_options.black_time as f64 - time_flex)
        }

        if time_for_move.is_none() {
            // wrong options, do not start timer
            return
        }

        let (tx, rx) = channel();
        self.timer_receiver = Some(rx);

        self.timer_guard = Some(self.timer.schedule_with_delay(
            Duration::milliseconds(time_for_move.unwrap() as i64), move || {
                tx.send(true).expect("Timeout message could not be sent.");
        }));
    }
}
