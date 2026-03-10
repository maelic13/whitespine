use std::sync::mpsc::Receiver;
use std::time::Instant;

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
    should_quit: bool,
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Engine {
        Engine {
            heuristic: Heuristic::default(),
            receiver,
            timer: None,
            time_for_move: f64::INFINITY,
            should_quit: false,
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
            self.search(
                &command.search_options.chess_game,
                command.search_options.search_depth(),
            );

            if self.should_quit {
                break;
            }
        }
    }

    fn initialize_heuristic(&mut self, search_options: &SearchOptions) {
        self.heuristic.fifty_moves_rule = search_options.fifty_moves_rule;
        self.heuristic.syzygy_path = search_options.syzygy_path.clone();
    }

    fn check_stop(&mut self) -> bool {
        if let Ok(command) = self.receiver.try_recv() {
            if command.quit {
                self.should_quit = true;
            }

            if command.stop || command.quit {
                return true;
            }
        }

        self.timer.unwrap().elapsed().as_millis() as f64 > self.time_for_move
    }

    fn search(&mut self, game: &Game, max_depth: f64) {
        let start = Instant::now();
        let possible_moves: Vec<ChessMove> = MoveGen::new_legal(&game.current_position()).collect();
        let mut moves: Vec<ChessMove> = possible_moves.first().copied().into_iter().collect();

        let mut depth: f64 = 0.;
        let mut evaluation: f64;
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
                (1_000_000. * nodes_searched as f64 / start.elapsed().as_micros().max(1) as f64)
                    as usize,
                start.elapsed().as_millis(),
                string_moves.join(" ")
            )
        }

        if let Some(best_move) = moves.first() {
            println!("bestmove {}", best_move);
        } else {
            println!("bestmove 0000");
        }
    }

    fn negamax(
        &mut self,
        game: &Game,
        depth: f64,
        mut alpha: f64,
        beta: f64,
    ) -> Result<(f64, Vec<ChessMove>, usize), &'static str> {
        if self.check_stop() {
            return Err("Calculation stopped.");
        }

        let mut nodes_searched: usize = 1;

        if game.result().is_some() {
            let result = game.result().unwrap();
            let color = game.side_to_move();
            return Ok((
                self.heuristic.evaluate_result(result, color),
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
                Err(message) => return Err(message),
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

    fn quiescence(
        &mut self,
        game: &Game,
        mut alpha: f64,
        beta: f64,
    ) -> Result<(f64, usize), &'static str> {
        if self.check_stop() {
            return Err("Calculation stopped.");
        }

        if game.result().is_some() {
            let result = game.result().unwrap();
            let color = game.side_to_move();
            return Ok((0.95 * self.heuristic.evaluate_result(result, color), 0));
        }
        if game.can_declare_draw() {
            return Ok((0.0, 0));
        }

        let in_check = game.current_position().checkers().0 != 0;
        let evaluation = 0.95 * self.heuristic.evaluate_position(game);

        if !in_check && evaluation >= beta {
            return Ok((beta, 0));
        }

        let use_delta_pruning = game
            .current_position()
            .combined()
            .collect::<Vec<Square>>()
            .len()
            > 8;
        let piece_value = PieceValue::default();

        if !in_check && use_delta_pruning && evaluation + piece_value.queen_value < alpha {
            return Ok((alpha, 0));
        }

        if !in_check && evaluation > alpha {
            alpha = evaluation;
        }

        let mut nodes_searched: usize = 0;
        for (chess_move, is_capture, is_en_passant) in self.get_quiescence_moves(game, in_check) {
            if !in_check
                && use_delta_pruning
                && is_en_passant
                && (evaluation + piece_value.pawn_value < alpha)
            {
                continue;
            } else if use_delta_pruning
                && !in_check
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
                Err(message) => return Err(message),
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

    fn get_quiescence_moves(&self, game: &Game, in_check: bool) -> Vec<(ChessMove, bool, bool)> {
        let mut captures_and_checks: Vec<(ChessMove, bool, bool)> = vec![];
        let legal_moves = MoveGen::new_legal(&game.current_position()).collect();
        let board = game.current_position();

        if in_check {
            return self
                .order_moves(&board, legal_moves)
                .into_iter()
                .map(|chess_move| {
                    let en_passant_capture = board.piece_on(chess_move.get_source()).unwrap()
                        == Piece::Pawn
                        && (chess_move.get_source().get_rank() != chess_move.get_dest().get_rank())
                        && (chess_move.get_source().get_file() != chess_move.get_dest().get_file());
                    let captured_piece =
                        board.piece_on(chess_move.get_dest()).is_some() || en_passant_capture;
                    (chess_move, captured_piece, en_passant_capture)
                })
                .collect();
        }

        let ordered_moves = self.order_moves(&board, legal_moves);

        for chess_move in ordered_moves {
            let board_after_move = board.make_move_new(chess_move);

            let captured_piece = board.piece_on(chess_move.get_dest()).is_some();
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
                self.time_for_move =
                    (0.05 * (white_time as f64 - search_options.move_overhead)).max(0.0);
            }
            (Color::White, _, white_time, white_increment, _, _) if white_time > 0 => {
                self.time_for_move = (0.1 * white_time as f64 + white_increment as f64
                    - search_options.move_overhead)
                    .min(white_time as f64 - search_options.move_overhead)
                    .max(0.0);
            }
            (Color::Black, _, _, _, black_time, 0) if black_time > 0 => {
                self.time_for_move =
                    (0.05 * (black_time as f64 - search_options.move_overhead)).max(0.0);
            }
            (Color::Black, _, _, _, black_time, black_increment) if black_time > 0 => {
                self.time_for_move = (0.1 * black_time as f64 + black_increment as f64
                    - search_options.move_overhead)
                    .min(black_time as f64 - search_options.move_overhead)
                    .max(0.0);
            }
            _ => panic!("Incorrect time options."),
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
            let is_en_passant = attacker == Some(Piece::Pawn)
                && from.get_rank() != to.get_rank()
                && from.get_file() != to.get_file()
                && victim.is_none();

            // MVV-LVA scoring
            if let (Some(att), Some(vic)) = (attacker, victim) {
                score += 10 * piece_value.get_piece_value(vic) as i32
                    - piece_value.get_piece_value(att) as i32;
            } else if is_en_passant {
                score += 9 * piece_value.pawn_value as i32;
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
