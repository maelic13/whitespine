use std::sync::mpsc::Receiver;
use std::time::Instant;

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};

use crate::engine_command::EngineCommand;
use crate::heuristic::Heuristic;
use crate::piece_value::PieceValue;
use crate::search_options::SearchOptions;

const MAX_PLY: usize = 128;
const MAX_SEARCH_DEPTH: usize = 64;
const TT_SIZE: usize = 1 << 20;
const INFINITY: i32 = 32_000;
const MATE_SCORE: i32 = 30_000;
const MATE_THRESHOLD: i32 = MATE_SCORE - MAX_PLY as i32;
const ASPIRATION_WINDOW: i32 = 40;

const TT_EXACT: u8 = 0;
const TT_LOWER: u8 = 1;
const TT_UPPER: u8 = 2;

#[derive(Clone, Copy)]
struct TTEntry {
    key: u64,
    depth: i16,
    score: i32,
    flag: u8,
    best_move: Option<ChessMove>,
}

impl TTEntry {
    fn empty() -> TTEntry {
        TTEntry {
            key: 0,
            depth: -1,
            score: 0,
            flag: TT_EXACT,
            best_move: None,
        }
    }
}

#[derive(Default)]
struct SearchStats {
    nodes: usize,
}

#[derive(Clone, Copy)]
struct ScoredMove {
    mv: ChessMove,
    score: i32,
    is_capture: bool,
    is_promotion: bool,
}

pub struct Engine {
    heuristic: Heuristic,
    receiver: Receiver<EngineCommand>,
    timer: Option<Instant>,
    time_for_move: f64,
    should_quit: bool,
    tt: Vec<TTEntry>,
    killers: [[Option<ChessMove>; 2]; MAX_PLY],
    history: [[[i32; 64]; 64]; 2],
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Engine {
        Engine {
            heuristic: Heuristic::default(),
            receiver,
            timer: None,
            time_for_move: f64::INFINITY,
            should_quit: false,
            tt: vec![TTEntry::empty(); TT_SIZE],
            killers: [[None; 2]; MAX_PLY],
            history: [[[0; 64]; 64]; 2],
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = self.receiver.recv().unwrap();

            if command.quit {
                break;
            }
            if command.stop {
                continue;
            }

            self.initialize_heuristic(&command.search_options);
            self.start_timer(&command.search_options);
            self.search(
                &command.search_options,
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
        self.killers = [[None; 2]; MAX_PLY];
        self.history = [[[0; 64]; 64]; 2];
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

    fn search(&mut self, search_options: &SearchOptions, max_depth: f64) {
        let start = Instant::now();
        let board = search_options.current_board;
        let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();

        if legal_moves.is_empty() {
            println!("bestmove 0000");
            return;
        }

        let mut best_move = legal_moves[0];
        let mut best_score = 0;
        let mut history = search_options.position_hash_history.clone();
        if history.last().copied() != Some(board.get_hash()) {
            history.push(board.get_hash());
        }
        let reversible_moves = search_options.reversible_moves;

        let max_depth = if max_depth.is_finite() {
            max_depth.max(1.0).min(MAX_SEARCH_DEPTH as f64) as usize
        } else {
            MAX_SEARCH_DEPTH
        };

        for depth in 1..=max_depth {
            let mut window = ASPIRATION_WINDOW;
            let mut alpha = if depth >= 4 {
                best_score - window
            } else {
                -INFINITY
            };
            let mut beta = if depth >= 4 {
                best_score + window
            } else {
                INFINITY
            };
            let (score, root_best_move, stats) = loop {
                let mut stats = SearchStats::default();
                let result = self.search_root(
                    board,
                    depth,
                    0,
                    alpha,
                    beta,
                    &mut history,
                    reversible_moves,
                    &mut stats,
                );

                let (score, root_best_move) = match result {
                    Ok(result) => result,
                    Err(_) => {
                        println!("bestmove {}", best_move);
                        return;
                    }
                };

                if score <= alpha && alpha != -INFINITY {
                    window *= 2;
                    alpha = (score - window).max(-INFINITY);
                    beta = (score + window).min(INFINITY);
                    continue;
                }
                if score >= beta && beta != INFINITY {
                    window *= 2;
                    alpha = (score - window).max(-INFINITY);
                    beta = (score + window).min(INFINITY);
                    continue;
                }

                break (score, root_best_move, stats);
            };

            best_score = score;
            best_move = root_best_move;
            let pv = self.extract_pv(board, depth);
            let pv_line = pv
                .iter()
                .map(|mv| mv.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let nps = (1_000_000.0 * stats.nodes as f64 / start.elapsed().as_micros().max(1) as f64)
                as usize;

            println!(
                "info depth {} score {} nodes {} nps {} time {} pv {}",
                depth,
                self.format_score(best_score),
                stats.nodes,
                nps,
                start.elapsed().as_millis(),
                pv_line
            );

            if self.check_stop() {
                break;
            }
        }

        println!("bestmove {}", best_move);
    }

    #[allow(clippy::too_many_arguments)]
    fn search_root(
        &mut self,
        board: Board,
        depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        history: &mut Vec<u64>,
        reversible_moves: usize,
        stats: &mut SearchStats,
    ) -> Result<(i32, ChessMove), &'static str> {
        let tt_move = self.tt_move(board);
        let mut moves = self.score_moves(board, ply, tt_move, false);
        if moves.is_empty() {
            return Ok((self.terminal_score(board, ply), ChessMove::default()));
        }

        let alpha_orig = alpha;
        let mut best_score = -INFINITY;
        let mut best_move = moves[0].mv;
        let in_check = board.checkers().0 != 0;

        for (index, scored_move) in moves.iter_mut().enumerate() {
            if self.check_stop() {
                return Err("Calculation stopped.");
            }

            let child = board.make_move_new(scored_move.mv);
            let gives_check = child.checkers().0 != 0;
            let extension = usize::from(in_check || gives_check);
            let next_depth = depth.saturating_sub(1) + extension.min(1);
            let irreversible =
                self.is_irreversible(board, child, scored_move.mv, scored_move.is_capture);
            let next_reversible = if irreversible {
                0
            } else {
                reversible_moves + 1
            };
            history.push(child.get_hash());

            let tactical = scored_move.is_capture || scored_move.is_promotion;
            let score = if index == 0 {
                -self.negamax(
                    child,
                    next_depth,
                    ply + 1,
                    -beta,
                    -alpha,
                    history,
                    next_reversible,
                    true,
                    stats,
                )?
            } else {
                let reduction =
                    if depth >= 3 && index >= 4 && !in_check && !tactical && !gives_check {
                        1 + usize::from(depth >= 6 && index >= 8)
                    } else {
                        0
                    };

                let reduced_depth = next_depth.saturating_sub(reduction);
                let mut score = -self.negamax(
                    child,
                    reduced_depth,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    history,
                    next_reversible,
                    true,
                    stats,
                )?;

                if reduction > 0 && score > alpha {
                    score = -self.negamax(
                        child,
                        next_depth,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        history,
                        next_reversible,
                        true,
                        stats,
                    )?;
                }
                if score > alpha && score < beta {
                    score = -self.negamax(
                        child,
                        next_depth,
                        ply + 1,
                        -beta,
                        -alpha,
                        history,
                        next_reversible,
                        true,
                        stats,
                    )?;
                }
                score
            };

            history.pop();

            if score > best_score {
                best_score = score;
                best_move = scored_move.mv;
            }
            if score > alpha {
                alpha = score;
            }
            if score >= beta {
                if !scored_move.is_capture && !scored_move.is_promotion {
                    self.record_killer(ply, scored_move.mv);
                    self.record_history(board.side_to_move(), scored_move.mv, depth);
                }
                self.store_tt(board, depth, ply, beta, TT_LOWER, Some(scored_move.mv));
                return Ok((beta, best_move));
            }
        }

        let flag = if best_score <= alpha_orig {
            TT_UPPER
        } else {
            TT_EXACT
        };
        self.store_tt(board, depth, ply, best_score, flag, Some(best_move));
        Ok((best_score, best_move))
    }

    #[allow(clippy::too_many_arguments)]
    fn negamax(
        &mut self,
        board: Board,
        depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        history: &mut Vec<u64>,
        reversible_moves: usize,
        allow_null: bool,
        stats: &mut SearchStats,
    ) -> Result<i32, &'static str> {
        if self.check_stop() {
            return Err("Calculation stopped.");
        }

        stats.nodes += 1;

        if self.is_draw(board, history, reversible_moves) {
            return Ok(0);
        }

        match board.status() {
            BoardStatus::Checkmate => return Ok(-MATE_SCORE + ply as i32),
            BoardStatus::Stalemate => return Ok(0),
            BoardStatus::Ongoing => {}
        }

        if ply >= MAX_PLY - 1 {
            return Ok(self.heuristic.evaluate_board(&board));
        }
        if depth == 0 {
            return self.quiescence(board, ply, alpha, beta, history, reversible_moves, stats);
        }

        let alpha_orig = alpha;
        let mut tt_move = None;
        if let Some(entry) = self.probe_tt(board.get_hash()) {
            tt_move = entry.best_move;
            if entry.depth >= depth as i16 {
                let score = self.score_from_tt(entry.score, ply);
                match entry.flag {
                    TT_EXACT => return Ok(score),
                    TT_LOWER => alpha = alpha.max(score),
                    TT_UPPER => {}
                    _ => {}
                }
                let beta = if entry.flag == TT_UPPER {
                    beta.min(score)
                } else {
                    beta
                };
                if alpha >= beta {
                    return Ok(score);
                }
            }
        }

        let in_check = board.checkers().0 != 0;
        let static_eval = self.heuristic.evaluate_board(&board);

        if allow_null
            && depth >= 3
            && !in_check
            && self.has_non_pawn_material(board, board.side_to_move())
            && static_eval >= beta
        {
            if let Some(null_board) = board.null_move() {
                history.push(null_board.get_hash());
                let score = -self.negamax(
                    null_board,
                    depth.saturating_sub(1 + 2),
                    ply + 1,
                    -beta,
                    -beta + 1,
                    history,
                    reversible_moves + 1,
                    false,
                    stats,
                )?;
                history.pop();

                if score >= beta {
                    return Ok(beta);
                }
            }
        }

        let mut moves = self.score_moves(board, ply, tt_move, false);
        if moves.is_empty() {
            return Ok(self.terminal_score(board, ply));
        }

        let mut best_score = -INFINITY;
        let mut best_move = None;

        for (index, scored_move) in moves.iter_mut().enumerate() {
            let child = board.make_move_new(scored_move.mv);
            let gives_check = child.checkers().0 != 0;
            let extension = usize::from(in_check || gives_check);
            let next_depth = depth.saturating_sub(1) + extension.min(1);
            let irreversible =
                self.is_irreversible(board, child, scored_move.mv, scored_move.is_capture);
            let next_reversible = if irreversible {
                0
            } else {
                reversible_moves + 1
            };
            history.push(child.get_hash());

            let tactical = scored_move.is_capture || scored_move.is_promotion;
            let score = if index == 0 {
                -self.negamax(
                    child,
                    next_depth,
                    ply + 1,
                    -beta,
                    -alpha,
                    history,
                    next_reversible,
                    true,
                    stats,
                )?
            } else {
                let reduction =
                    if depth >= 3 && index >= 4 && !in_check && !tactical && !gives_check {
                        1 + usize::from(depth >= 6 && index >= 8)
                    } else {
                        0
                    };

                let reduced_depth = next_depth.saturating_sub(reduction);
                let mut score = -self.negamax(
                    child,
                    reduced_depth,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    history,
                    next_reversible,
                    true,
                    stats,
                )?;

                if reduction > 0 && score > alpha {
                    score = -self.negamax(
                        child,
                        next_depth,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        history,
                        next_reversible,
                        true,
                        stats,
                    )?;
                }
                if score > alpha && score < beta {
                    score = -self.negamax(
                        child,
                        next_depth,
                        ply + 1,
                        -beta,
                        -alpha,
                        history,
                        next_reversible,
                        true,
                        stats,
                    )?;
                }
                score
            };

            history.pop();

            if score > best_score {
                best_score = score;
                best_move = Some(scored_move.mv);
            }
            if score > alpha {
                alpha = score;
            }
            if score >= beta {
                if !scored_move.is_capture && !scored_move.is_promotion {
                    self.record_killer(ply, scored_move.mv);
                    self.record_history(board.side_to_move(), scored_move.mv, depth);
                }
                self.store_tt(board, depth, ply, beta, TT_LOWER, Some(scored_move.mv));
                return Ok(beta);
            }
        }

        let flag = if best_score <= alpha_orig {
            TT_UPPER
        } else {
            TT_EXACT
        };
        self.store_tt(board, depth, ply, best_score, flag, best_move);
        Ok(best_score)
    }

    #[allow(clippy::too_many_arguments)]
    fn quiescence(
        &mut self,
        board: Board,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        history: &mut Vec<u64>,
        reversible_moves: usize,
        stats: &mut SearchStats,
    ) -> Result<i32, &'static str> {
        if self.check_stop() {
            return Err("Calculation stopped.");
        }

        stats.nodes += 1;

        if self.is_draw(board, history, reversible_moves) {
            return Ok(0);
        }

        match board.status() {
            BoardStatus::Checkmate => return Ok(-MATE_SCORE + ply as i32),
            BoardStatus::Stalemate => return Ok(0),
            BoardStatus::Ongoing => {}
        }

        let in_check = board.checkers().0 != 0;
        let stand_pat = self.heuristic.evaluate_board(&board);

        if !in_check {
            if stand_pat >= beta {
                return Ok(beta);
            }
            alpha = alpha.max(stand_pat);
        }

        let mut moves = self.score_moves(board, ply, self.tt_move(board), true);
        if moves.is_empty() {
            return Ok(alpha);
        }

        for scored_move in &mut moves {
            if !in_check && scored_move.is_capture {
                let gain = self.capture_value(board, scored_move.mv) + 150;
                if stand_pat + gain < alpha {
                    continue;
                }
            }

            let child = board.make_move_new(scored_move.mv);
            let irreversible =
                self.is_irreversible(board, child, scored_move.mv, scored_move.is_capture);
            let next_reversible = if irreversible {
                0
            } else {
                reversible_moves + 1
            };
            history.push(child.get_hash());
            let score = -self.quiescence(
                child,
                ply + 1,
                -beta,
                -alpha,
                history,
                next_reversible,
                stats,
            )?;
            history.pop();

            if score >= beta {
                return Ok(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }

        Ok(alpha)
    }

    fn score_moves(
        &self,
        board: Board,
        ply: usize,
        tt_move: Option<ChessMove>,
        quiescence: bool,
    ) -> Vec<ScoredMove> {
        let side = board.side_to_move().to_index();
        let mut scored_moves = Vec::new();
        let piece_value = PieceValue::default();
        let in_check = board.checkers().0 != 0;

        for mv in MoveGen::new_legal(&board) {
            let is_capture = self.is_capture(board, mv);
            let is_promotion = mv.get_promotion().is_some();

            if quiescence && !in_check && !is_capture && !is_promotion {
                continue;
            }

            let mut score =
                self.history[side][mv.get_source().to_index()][mv.get_dest().to_index()];

            if Some(mv) == tt_move {
                score += 2_000_000;
            } else if is_capture {
                let victim = self.capture_value(board, mv);
                let attacker = board
                    .piece_on(mv.get_source())
                    .map(|piece| piece_value.get_piece_value(piece) as i32)
                    .unwrap_or(0);
                score += 100_000 + 10 * victim - attacker;
            } else if is_promotion {
                score += 90_000
                    + mv.get_promotion()
                        .map(|piece| piece_value.get_piece_value(piece) as i32)
                        .unwrap_or(0);
            } else if self.killers[ply][0] == Some(mv) {
                score += 80_000;
            } else if self.killers[ply][1] == Some(mv) {
                score += 79_000;
            }

            scored_moves.push(ScoredMove {
                mv,
                score,
                is_capture,
                is_promotion,
            });
        }

        scored_moves.sort_by(|a, b| b.score.cmp(&a.score));
        scored_moves
    }

    fn extract_pv(&self, board: Board, depth: usize) -> Vec<ChessMove> {
        let mut pv = Vec::new();
        let mut current = board;
        let mut seen_hashes = Vec::new();

        for _ in 0..depth {
            let hash = current.get_hash();
            if seen_hashes.contains(&hash) {
                break;
            }
            seen_hashes.push(hash);

            let Some(entry) = self.probe_tt(hash) else {
                break;
            };
            let Some(mv) = entry.best_move else {
                break;
            };

            if !MoveGen::new_legal(&current).any(|legal| legal == mv) {
                break;
            }

            pv.push(mv);
            current = current.make_move_new(mv);
        }

        pv
    }

    fn tt_move(&self, board: Board) -> Option<ChessMove> {
        self.probe_tt(board.get_hash())
            .and_then(|entry| entry.best_move)
    }

    fn probe_tt(&self, key: u64) -> Option<TTEntry> {
        let entry = self.tt[key as usize % self.tt.len()];
        if entry.depth >= 0 && entry.key == key {
            Some(entry)
        } else {
            None
        }
    }

    fn store_tt(
        &mut self,
        board: Board,
        depth: usize,
        ply: usize,
        score: i32,
        flag: u8,
        best_move: Option<ChessMove>,
    ) {
        let key = board.get_hash();
        let index = key as usize % self.tt.len();
        let stored_score = self.score_to_tt(score, ply);
        let entry = self.tt[index];

        if entry.depth <= depth as i16 || entry.key != key {
            self.tt[index] = TTEntry {
                key,
                depth: depth as i16,
                score: stored_score,
                flag,
                best_move,
            };
        }
    }

    fn score_to_tt(&self, score: i32, ply: usize) -> i32 {
        if score > MATE_THRESHOLD {
            score + ply as i32
        } else if score < -MATE_THRESHOLD {
            score - ply as i32
        } else {
            score
        }
    }

    fn score_from_tt(&self, score: i32, ply: usize) -> i32 {
        if score > MATE_THRESHOLD {
            score - ply as i32
        } else if score < -MATE_THRESHOLD {
            score + ply as i32
        } else {
            score
        }
    }

    fn terminal_score(&self, board: Board, ply: usize) -> i32 {
        match board.status() {
            BoardStatus::Checkmate => -MATE_SCORE + ply as i32,
            BoardStatus::Stalemate => 0,
            BoardStatus::Ongoing => 0,
        }
    }

    fn is_draw(&self, board: Board, history: &[u64], reversible_moves: usize) -> bool {
        if self.heuristic.fifty_moves_rule && reversible_moves >= 100 {
            return true;
        }

        let window = reversible_moves.min(history.len().saturating_sub(1)) + 1;
        let slice = &history[history.len() - window..];
        let current_hash = board.get_hash();
        slice.iter().filter(|&&hash| hash == current_hash).count() >= 3
    }

    fn has_non_pawn_material(&self, board: Board, color: Color) -> bool {
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            if (*board.pieces(piece) & *board.color_combined(color)).popcnt() != 0 {
                return true;
            }
        }
        false
    }

    fn is_capture(&self, board: Board, mv: ChessMove) -> bool {
        board.piece_on(mv.get_dest()).is_some() || self.is_en_passant(board, mv)
    }

    fn is_en_passant(&self, board: Board, mv: ChessMove) -> bool {
        board.piece_on(mv.get_source()) == Some(Piece::Pawn)
            && mv.get_source().get_file() != mv.get_dest().get_file()
            && board.piece_on(mv.get_dest()).is_none()
    }

    fn capture_value(&self, board: Board, mv: ChessMove) -> i32 {
        let piece_value = PieceValue::default();
        if self.is_en_passant(board, mv) {
            piece_value.pawn_value as i32
        } else {
            board
                .piece_on(mv.get_dest())
                .map(|piece| piece_value.get_piece_value(piece) as i32)
                .unwrap_or(0)
        }
    }

    fn is_irreversible(&self, board: Board, child: Board, mv: ChessMove, is_capture: bool) -> bool {
        board.piece_on(mv.get_source()) == Some(Piece::Pawn)
            || is_capture
            || child.castle_rights(Color::White) != board.castle_rights(Color::White)
            || child.castle_rights(Color::Black) != board.castle_rights(Color::Black)
    }

    fn record_killer(&mut self, ply: usize, mv: ChessMove) {
        if self.killers[ply][0] != Some(mv) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(mv);
        }
    }

    fn record_history(&mut self, color: Color, mv: ChessMove, depth: usize) {
        let bonus = (depth * depth) as i32;
        let entry = &mut self.history[color.to_index()][mv.get_source().to_index()]
            [mv.get_dest().to_index()];
        *entry = (*entry + bonus).min(200_000);
    }

    fn start_timer(&mut self, search_options: &SearchOptions) {
        self.timer = Some(Instant::now());
        self.time_for_move = f64::INFINITY;

        match (
            search_options.current_board.side_to_move(),
            search_options.move_time,
            search_options.white_time,
            search_options.white_increment,
            search_options.black_time,
            search_options.black_increment,
        ) {
            (_, 0, 0, 0, 0, 0) => {}
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

    fn format_score(&self, score: i32) -> String {
        if score > MATE_THRESHOLD {
            format!("mate {}", (MATE_SCORE - score + 1) / 2)
        } else if score < -MATE_THRESHOLD {
            format!("mate -{}", (MATE_SCORE + score + 1) / 2)
        } else {
            format!("cp {}", score)
        }
    }
}
