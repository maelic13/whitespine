use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chess::{Board, ChessMove, Color, Game, MoveGen, Piece, Square};

use crate::engine_command::EngineCommand;
use crate::heuristic::Heuristic;
use crate::piece_value::PieceValue;
use crate::search_options::SearchOptions;

pub struct Engine {
    heuristic: Heuristic,
    receiver: Receiver<EngineCommand>,
    timer: Option<Instant>,
    time_for_move: f64,
    pondering: bool,
    ponderhit_seen: bool,
    bestmove_released: bool,
}

enum SearchControl {
    Stop,
    Quit,
}

impl SearchControl {
    fn should_quit(&self) -> bool {
        matches!(self, SearchControl::Quit)
    }
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Engine {
        Engine {
            heuristic: Heuristic::default(),
            receiver,
            timer: None,
            time_for_move: f64::INFINITY,
            pondering: false,
            ponderhit_seen: false,
            bestmove_released: false,
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = match self.receiver.recv() {
                Ok(command) => command,
                Err(_) => {
                    println!("info string Command channel closed.");
                    break;
                }
            };

            if command.quit {
                break;
            } else if command.stop || command.ponderhit {
                continue;
            }

            self.start_timer(&command.search_options);
            if self.search(&command.search_options).should_quit() {
                break;
            }
        }
    }

    fn check_stop(&mut self) -> Result<(), SearchControl> {
        match self.receiver.try_recv() {
            Ok(command) if command.quit => {
                self.bestmove_released = true;
                Err(SearchControl::Quit)
            }
            Ok(command) if command.stop => {
                self.bestmove_released = true;
                Err(SearchControl::Stop)
            }
            Ok(command) if command.ponderhit => {
                self.ponderhit_seen = true;
                self.pondering = false;
                self.timer = Some(Instant::now());
                Ok(())
            }
            Ok(_) => Ok(()),
            Err(TryRecvError::Disconnected) => {
                self.bestmove_released = true;
                Err(SearchControl::Quit)
            }
            _ if self.timer_expired() => Err(SearchControl::Stop),
            _ => Ok(()),
        }
    }

    fn search(&mut self, search_options: &SearchOptions) -> SearchControl {
        let game = &search_options.chess_game;
        let depth_limit = search_options.depth;
        let start = Instant::now();
        self.pondering = search_options.ponder;
        self.ponderhit_seen = false;
        self.bestmove_released = false;

        if game.result().is_some() || game.can_declare_draw() {
            let control = self.wait_until_bestmove_allowed(search_options);
            println!("bestmove 0000");
            return control;
        }

        // start with random move choice, to be used in case of timeout before first depth is reached
        let possible_moves: Vec<ChessMove> = MoveGen::new_legal(&game.current_position()).collect();
        let mut moves: Vec<ChessMove> = if possible_moves.is_empty() {
            Vec::new()
        } else {
            vec![possible_moves[(start.elapsed().as_nanos() / 100) as usize % possible_moves.len()]]
        };

        let mut depth: f64 = 0.;
        let mut evaluation: f64;
        let mut nodes_searched: usize = 0;
        let mut control = SearchControl::Stop;

        while depth < depth_limit {
            depth += 1.;

            let result = self.negamax(&game, depth, f64::NEG_INFINITY, f64::INFINITY);
            match result {
                Ok((eval, pv, nodes)) => {
                    evaluation = eval;
                    nodes_searched += nodes;
                    moves = pv;
                }
                Err(stop_reason) => {
                    control = stop_reason;
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
                (1_000_000. * nodes_searched as f64 / start.elapsed().as_micros() as f64) as usize,
                start.elapsed().as_millis(),
                string_moves.join(" ")
            )
        }

        if !self.bestmove_released {
            let delayed_control = self.wait_until_bestmove_allowed(search_options);
            if delayed_control.should_quit() {
                control = delayed_control;
            }
        }

        let bestmove = Self::legal_bestmove_or_fallback(&game.current_position(), &moves);
        println!(
            "bestmove {}",
            bestmove
                .map(|chess_move| chess_move.to_string())
                .unwrap_or_else(|| String::from("0000"))
        );
        control
    }

    fn wait_until_bestmove_allowed(&mut self, search_options: &SearchOptions) -> SearchControl {
        if !search_options.ponder && !search_options.infinite {
            return SearchControl::Stop;
        }

        while !(search_options.ponder && self.ponderhit_seen) {
            match self.receiver.try_recv() {
                Ok(command) if command.quit => return SearchControl::Quit,
                Ok(command) if command.stop => return SearchControl::Stop,
                Ok(command) if command.ponderhit => {
                    self.ponderhit_seen = true;
                    self.pondering = false;
                    self.timer = Some(Instant::now());
                    if search_options.ponder {
                        return SearchControl::Stop;
                    }
                }
                Ok(_) | Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(1)),
                Err(TryRecvError::Disconnected) => return SearchControl::Quit,
            }
        }

        SearchControl::Stop
    }

    fn timer_expired(&self) -> bool {
        if self.pondering && !self.ponderhit_seen {
            return false;
        }
        self.timer
            .map(|timer| timer.elapsed().as_millis() as f64 > self.time_for_move)
            .unwrap_or(false)
    }

    fn legal_bestmove_or_fallback(
        board: &Board,
        principal_variation: &[ChessMove],
    ) -> Option<ChessMove> {
        let legal_moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
        if legal_moves.is_empty() {
            return None;
        }
        if let Some(bestmove) = principal_variation.first()
            && legal_moves.contains(bestmove)
        {
            return Some(*bestmove);
        }
        legal_moves.first().copied()
    }

    fn negamax(
        &mut self,
        game: &Game,
        depth: f64,
        mut alpha: f64,
        beta: f64,
    ) -> Result<(f64, Vec<ChessMove>, usize), SearchControl> {
        self.check_stop()?;

        let mut nodes_searched: usize = 1;

        if game.result().is_some() {
            let result = game.result().unwrap();
            let color = game.side_to_move();
            return Ok((
                self.heuristic.evaluate_result(result, color, depth),
                vec![],
                nodes_searched,
            ));
        }
        if game.can_declare_draw() {
            return Ok((0.0, vec![], nodes_searched));
        }
        if depth == 0. {
            let evaluation: f64;
            let result = self.quiescence(game, alpha, beta);
            match result {
                Ok((eval, nodes)) => {
                    evaluation = eval;
                    nodes_searched += nodes;
                }
                Err(control) => return Err(control),
            }
            return Ok((evaluation, vec![], nodes_searched));
        }

        let legal_moves = MoveGen::new_legal(&game.current_position()).collect();
        let ordered_moves = self.order_moves(&game.current_position(), legal_moves);
        let mut best_moves: Vec<ChessMove> = vec![];
        let mut moves: Vec<ChessMove>;
        let mut current_game: Game;
        let mut evaluation: f64;

        for chess_move in ordered_moves {
            current_game = game.clone();
            current_game.make_move(chess_move);

            let result = self.negamax(&current_game, depth - 1., -beta, -alpha);
            match result {
                Ok((eval, pv, nodes)) => {
                    evaluation = eval;
                    nodes_searched += nodes;
                    moves = pv;
                }
                Err(control) => return Err(control),
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

    fn quiescence(
        &mut self,
        game: &Game,
        mut alpha: f64,
        beta: f64,
    ) -> Result<(f64, usize), SearchControl> {
        self.check_stop()?;

        if game.result().is_some() {
            let result = game.result().unwrap();
            let color = game.side_to_move();
            return Ok((0.95 * self.heuristic.evaluate_result(result, color, 0.), 0));
        }
        if game.can_declare_draw() {
            return Ok((0.0, 0));
        }

        let evaluation = 0.95 * self.heuristic.evaluate_position(game);

        if evaluation >= beta {
            return Ok((beta, 0));
        }

        let use_delta_pruning = game
            .current_position()
            .combined()
            .collect::<Vec<Square>>()
            .len()
            > 8;
        let piece_value = PieceValue::default();

        if use_delta_pruning && evaluation < alpha - piece_value.queen_value {
            return Ok((alpha, 0));
        }

        if evaluation > alpha {
            alpha = evaluation;
        }

        let mut nodes_searched: usize = 0;
        for (chess_move, is_capture, is_en_passant) in self.get_captures_and_checks(&game) {
            if use_delta_pruning && is_en_passant && (evaluation + piece_value.pawn_value < alpha) {
                continue;
            } else if use_delta_pruning
                && is_capture
                && (evaluation
                    + piece_value.get_piece_value(
                        game.current_position()
                            .piece_on(chess_move.get_dest())
                            .unwrap(),
                    )
                    + piece_value.pawn_value
                    < alpha)
            {
                continue;
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
                Err(control) => return Err(control),
            }

            if score >= beta {
                return Ok((beta, nodes_searched));
            }
            if score > alpha {
                alpha = score;
            }
        }

        Ok((alpha, nodes_searched))
    }

    fn get_captures_and_checks(&self, game: &Game) -> Vec<(ChessMove, bool, bool)> {
        let mut captures_and_checks: Vec<(ChessMove, bool, bool)> = vec![];
        let legal_moves = MoveGen::new_legal(&game.current_position()).collect();
        let ordered_moves = self.order_moves(&game.current_position(), legal_moves);
        let board = game.current_position();

        for chess_move in ordered_moves {
            let board_after_move = board.make_move_new(chess_move);

            let captured_piece = board.piece_on(chess_move.get_dest()) != None;
            let is_check = board_after_move.checkers().collect::<Vec<Square>>().len() != 0;

            let en_passant_capture = board.piece_on(chess_move.get_source()).unwrap()
                == Piece::Pawn
                && (chess_move.get_source().get_rank() != chess_move.get_dest().get_rank())
                && (chess_move.get_source().get_file() != chess_move.get_dest().get_file());

            if captured_piece || en_passant_capture || is_check {
                captures_and_checks.push((chess_move, captured_piece, en_passant_capture));
            }
        }

        captures_and_checks
    }

    fn start_timer(&mut self, search_options: &SearchOptions) {
        /* Start timer to check elapsed time and stop it over limit. */
        self.timer = Some(Instant::now());
        self.time_for_move = f64::INFINITY;

        match (
            search_options.chess_game.side_to_move(),
            search_options.move_time,
            search_options.white_time,
            search_options.white_increment,
            search_options.black_time,
            search_options.black_increment,
        ) {
            (_, 0, 0, 0, 0, 0) => return,
            (_, move_time, _, _, _, _) if move_time > 0 => {
                self.time_for_move = move_time as f64;
            }
            (Color::White, _, white_time, 0, _, _) if white_time > 0 => {
                self.time_for_move = 0.05 * (white_time as f64 - search_options.move_overhead);
            }
            (Color::White, _, white_time, white_increment, _, _) if white_time > 0 => {
                self.time_for_move = (0.1 * white_time as f64 + white_increment as f64
                    - search_options.move_overhead)
                    .min(white_time as f64 - search_options.move_overhead);
            }
            (Color::Black, _, _, _, black_time, 0) if black_time > 0 => {
                self.time_for_move = 0.05 * (black_time as f64 - search_options.move_overhead);
            }
            (Color::Black, _, _, _, black_time, black_increment) if black_time > 0 => {
                self.time_for_move = (0.1 * black_time as f64 + black_increment as f64
                    - search_options.move_overhead)
                    .min(black_time as f64 - search_options.move_overhead);
            }
            _ => return,
        }
    }

    fn order_moves(&self, board: &Board, moves: Vec<ChessMove>) -> Vec<ChessMove> {
        let mut scored_moves: Vec<(ChessMove, i32)> = vec![];
        let piece_value = PieceValue::default();

        for mv in moves {
            let mut score = 0;
            let from = mv.get_source();
            let to = mv.get_dest();
            let attacker = board.piece_on(from);
            let victim = board.piece_on(to);

            // MVV-LVA scoring
            if let (Some(att), Some(vic)) = (attacker, victim) {
                score += 10 * piece_value.get_piece_value(vic) as i32
                    - piece_value.get_piece_value(att) as i32;
            }

            // Promotion bonus
            if let Some(promo) = mv.get_promotion() {
                score += 5 * piece_value.get_piece_value(promo) as i32;
            }

            // Check bonus
            let new_board = board.make_move_new(mv);
            if new_board.checkers().0 != 0 {
                score += 1;
            }

            scored_moves.push((mv, score));
        }

        scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
        scored_moves.into_iter().map(|(mv, _)| mv).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::sync::mpsc::channel;

    #[test]
    fn bestmove_output_falls_back_to_a_legal_move() {
        let board = Board::from_str("8/6K1/8/8/8/p7/P7/1k6 b - - 4 71").unwrap();
        let illegal_for_position = ChessMove::from_str("e2e4").unwrap();

        let bestmove = Engine::legal_bestmove_or_fallback(&board, &[illegal_for_position])
            .expect("final position must have legal black moves");

        assert_ne!(bestmove, illegal_for_position);
        assert!(MoveGen::new_legal(&board).any(|legal| legal == bestmove));
    }

    #[test]
    fn bestmove_output_is_0000_when_no_legal_move_exists() {
        let board = Board::from_str("7k/5Q2/7K/8/8/8/8/8 b - - 0 1").unwrap();

        assert!(Engine::legal_bestmove_or_fallback(&board, &[]).is_none());
    }

    #[test]
    fn ponder_search_does_not_timeout_before_ponderhit() {
        let (_sender, receiver) = channel();
        let mut engine = Engine::new(receiver);
        engine.timer = Some(Instant::now() - Duration::from_millis(50));
        engine.time_for_move = 0.0;
        engine.pondering = true;
        engine.ponderhit_seen = false;

        assert!(!engine.timer_expired());

        engine.pondering = false;
        engine.ponderhit_seen = true;
        assert!(engine.timer_expired());
    }

    #[test]
    fn ponderhit_resets_the_move_timer() {
        let (sender, receiver) = channel();
        let mut engine = Engine::new(receiver);
        engine.timer = Some(Instant::now() - Duration::from_millis(50));
        engine.time_for_move = 1.0;
        engine.pondering = true;

        sender.send(EngineCommand::ponderhit()).unwrap();

        assert!(engine.check_stop().is_ok());
        assert!(!engine.pondering);
        assert!(engine.ponderhit_seen);
        assert!(!engine.timer_expired());
    }
}
