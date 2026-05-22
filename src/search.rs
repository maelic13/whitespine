use std::time::Instant;

use crate::board::board::Board;
use crate::board::movegen::{generate_captures, generate_legal_moves};
use crate::board::moves::Move;
use crate::board::piece::Piece;
use crate::eval::Evaluator;
use crate::tt::{INF_EVAL, MATE_SCORE, MAX_PLY, TTFlag, TranspositionTable, VALUE_NONE};

const MAX_MOVES: usize = 256;
const HISTORY_MAX: i32 = 16384;
const CORR_HIST_MAX: i32 = 16384;
const ASPIRATION_WINDOW: i32 = 25;

#[derive(Copy, Clone)]
struct SearchStack {
    mv: Move,
    excluded: Move,
    killers: [Move; 2],
    eval: i32,
    moved_piece: u8,
}

impl Default for SearchStack {
    fn default() -> Self {
        Self {
            mv: Move::NULL,
            excluded: Move::NULL,
            killers: [Move::NULL; 2],
            eval: VALUE_NONE,
            moved_piece: 255,
        }
    }
}

pub struct SearchInfo {
    pub depth: u32,
    pub seldepth: u32,
    pub score: i32,
    pub nodes: u64,
    pub time_ms: u64,
    pub nps: u64,
    pub hashfull: usize,
    pub pv: Vec<Move>,
    pub best_move: Move,
}

pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
    pub depth: u32,
    pub nodes: u64,
    pub time_ms: u64,
    pub pv: Vec<Move>,
    pub stability: u32,
    pub stopped: bool,
}

pub struct Searcher {
    main_hist: Box<[[i16; 64]; 64 * 2]>,
    cap_hist: Box<[[[i16; 6]; 64]; 6]>,
    cont_hist1: Box<[[[[i16; 64]; 6]; 64]; 6]>,
    cont_hist2: Box<[[[[i16; 64]; 6]; 64]; 6]>,
    countermove: Box<[[Move; 64]; 64]>,
    corr_hist: Box<[[i32; 16384]; 2]>,
    lmr_table: [[i32; 64]; 64],
    nodes: u64,
    seldepth: usize,
    stack: [SearchStack; MAX_PLY + 8],
    start_time: Instant,
    soft_limit: u64,
    hard_limit: u64,
    stop: bool,
    pub evaluator: Evaluator,
}

impl Searcher {
    pub fn new() -> Self {
        let mut lmr = [[0i32; 64]; 64];
        for depth in 1..64 {
            for mv in 1..64 {
                lmr[depth][mv] = (0.75 + (depth as f64).ln() * (mv as f64).ln() / 2.25) as i32;
            }
        }
        Self {
            main_hist: vec![[0i16; 64]; 128]
                .into_boxed_slice()
                .try_into()
                .expect("main history size"),
            cap_hist: vec![[[0i16; 6]; 64]; 6]
                .into_boxed_slice()
                .try_into()
                .expect("capture history size"),
            cont_hist1: vec![[[[0i16; 64]; 6]; 64]; 6]
                .into_boxed_slice()
                .try_into()
                .expect("continuation history size"),
            cont_hist2: vec![[[[0i16; 64]; 6]; 64]; 6]
                .into_boxed_slice()
                .try_into()
                .expect("continuation history size"),
            countermove: vec![[Move::NULL; 64]; 64]
                .into_boxed_slice()
                .try_into()
                .expect("countermove size"),
            corr_hist: vec![[0i32; 16384]; 2]
                .into_boxed_slice()
                .try_into()
                .expect("correction history size"),
            lmr_table: lmr,
            nodes: 0,
            seldepth: 0,
            stack: std::array::from_fn(|_| SearchStack::default()),
            start_time: Instant::now(),
            soft_limit: u64::MAX,
            hard_limit: u64::MAX,
            stop: false,
            evaluator: Evaluator::new(),
        }
    }

    pub fn clear_history(&mut self) {
        for row in self.main_hist.iter_mut() {
            for value in row.iter_mut() {
                *value /= 2;
            }
        }
        for attacker in self.cap_hist.iter_mut() {
            for target in attacker.iter_mut() {
                for value in target.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for hist in [&mut self.cont_hist1, &mut self.cont_hist2] {
            for prev_piece in hist.iter_mut() {
                for prev_to in prev_piece.iter_mut() {
                    for piece in prev_to.iter_mut() {
                        for value in piece.iter_mut() {
                            *value = 0;
                        }
                    }
                }
            }
        }
        for side in self.corr_hist.iter_mut() {
            for value in side.iter_mut() {
                *value /= 2;
            }
        }
    }

    pub fn clear_all(&mut self) {
        for row in self.main_hist.iter_mut() {
            for value in row.iter_mut() {
                *value = 0;
            }
        }
        for attacker in self.cap_hist.iter_mut() {
            for target in attacker.iter_mut() {
                for value in target.iter_mut() {
                    *value = 0;
                }
            }
        }
        for hist in [&mut self.cont_hist1, &mut self.cont_hist2] {
            for prev_piece in hist.iter_mut() {
                for prev_to in prev_piece.iter_mut() {
                    for piece in prev_to.iter_mut() {
                        for value in piece.iter_mut() {
                            *value = 0;
                        }
                    }
                }
            }
        }
        *self.countermove = [[Move::NULL; 64]; 64];
        for side in self.corr_hist.iter_mut() {
            for value in side.iter_mut() {
                *value = 0;
            }
        }
    }

    pub fn iterative_deepening<F, G>(
        &mut self,
        board: &mut Board,
        tt: &mut TranspositionTable,
        depth_limit: u32,
        soft_limit: u64,
        hard_limit: u64,
        mut should_stop: F,
        mut on_iteration: G,
    ) -> SearchResult
    where
        F: FnMut() -> bool,
        G: FnMut(SearchInfo),
    {
        self.nodes = 0;
        self.seldepth = 0;
        self.stack.fill(SearchStack::default());
        self.start_time = Instant::now();
        self.soft_limit = soft_limit;
        self.hard_limit = hard_limit;
        self.stop = false;
        self.clear_history();
        tt.new_search();

        let max_depth = depth_limit.min((MAX_PLY - 1) as u32).max(1);
        let mut best_move = Move::NULL;
        let mut best_score: i32 = 0;
        let mut best_pv = Vec::new();
        let mut completed_depth = 0u32;
        let mut stability = 0u32;
        let mut last_best = Move::NULL;

        for depth in 1..=max_depth {
            let mut alpha = -INF_EVAL;
            let mut beta = INF_EVAL;
            let mut window = ASPIRATION_WINDOW;
            if depth >= 4
                && completed_depth > 0
                && best_score.abs() < MATE_SCORE - MAX_PLY as i32
            {
                alpha = (best_score - window).max(-INF_EVAL);
                beta = (best_score + window).min(INF_EVAL);
            }

            self.stack[0].mv = best_move;
            let score = loop {
                let score = self.negamax(
                    board,
                    tt,
                    depth as i32,
                    alpha,
                    beta,
                    0,
                    false,
                    &mut should_stop,
                );
                if self.stop {
                    break score;
                }
                if score <= alpha {
                    // Fail low: widen alpha, keep beta
                    beta = (alpha + beta) / 2;
                    alpha = (score - window).max(-INF_EVAL);
                    window *= 2;
                    if window >= 900 {
                        alpha = -INF_EVAL;
                        beta = INF_EVAL;
                    }
                    continue;
                }
                if score >= beta {
                    // Fail high: widen beta, keep alpha
                    beta = (score + window).min(INF_EVAL);
                    window *= 2;
                    if window >= 900 {
                        alpha = -INF_EVAL;
                        beta = INF_EVAL;
                    }
                    continue;
                }
                break score;
            };

            if self.stop {
                break;
            }

            let current_best = if self.stack[0].mv.is_null() {
                tt.probe(board.hash)
                    .map(|entry| Move(entry.mv))
                    .filter(|mv| !mv.is_null())
                    .unwrap_or(best_move)
            } else {
                self.stack[0].mv
            };

            if current_best == last_best {
                stability += 1;
            } else {
                stability = 0;
                last_best = current_best;
            }

            best_move = current_best;
            best_score = score;
            completed_depth = depth;
            best_pv = self.extract_pv(board, tt, depth as usize);
            if best_move.is_null() && !best_pv.is_empty() {
                best_move = best_pv[0];
            }

            let time_ms = self.elapsed_ms();
            let nps = if time_ms > 0 {
                self.nodes * 1000 / time_ms
            } else {
                self.nodes
            };
            on_iteration(SearchInfo {
                depth,
                seldepth: self.seldepth as u32,
                score,
                nodes: self.nodes,
                time_ms,
                nps,
                hashfull: tt.hashfull(),
                pv: best_pv.clone(),
                best_move,
            });

            if time_ms >= self.dynamic_soft_limit(stability) {
                break;
            }
        }

        SearchResult {
            best_move,
            score: best_score,
            depth: completed_depth,
            nodes: self.nodes,
            time_ms: self.elapsed_ms(),
            pv: best_pv,
            stability,
            stopped: self.stop,
        }
    }

    fn negamax<F>(
        &mut self,
        board: &mut Board,
        tt: &mut TranspositionTable,
        mut depth: i32,
        mut alpha: i32,
        mut beta: i32,
        ply: usize,
        no_nmp: bool,
        should_stop: &mut F,
    ) -> i32
    where
        F: FnMut() -> bool,
    {
        self.nodes += 1;
        self.seldepth = self.seldepth.max(ply);
        if self.check_stop(should_stop) {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return self.evaluator.evaluate(board);
        }
        if board.is_draw() {
            return 0;
        }

        let pv_node = beta - alpha > 1;
        let original_alpha = alpha;
        let in_check = board.is_in_check();

        // Check extension at root of negamax (basilisk-style)
        if in_check && self.stack[ply].excluded.is_null() && ply < MAX_PLY - 2 {
            depth += 1;
        }

        alpha = alpha.max(-MATE_SCORE + ply as i32);
        beta = beta.min(MATE_SCORE - ply as i32 - 1);
        if alpha >= beta {
            return alpha;
        }
        if depth <= 0 {
            return self.quiescence(board, tt, alpha, beta, ply, should_stop);
        }

        let tt_entry = tt.probe(board.hash);
        let tt_move = tt_entry.map(|entry| Move(entry.mv)).unwrap_or(Move::NULL);
        if let Some(entry) = tt_entry {
            if !pv_node && self.stack[ply].excluded.is_null() && entry.depth as i32 >= depth {
                let tt_score = TranspositionTable::score_from_tt(entry.score as i32, ply);
                match entry.flag() {
                    TTFlag::Exact => return tt_score,
                    TTFlag::Alpha if tt_score <= alpha => return tt_score,
                    TTFlag::Beta if tt_score >= beta => return tt_score,
                    _ => {}
                }
            }
        }

        let corr_index = (board.pawn_hash() as usize) & 16383;
        let corr = self.corr_hist[board.side_to_move as usize][corr_index] / 16;
        let raw_static = if !self.stack[ply].excluded.is_null() {
            // Inherit eval from parent during singular search
            self.stack[ply].eval
        } else if in_check {
            VALUE_NONE
        } else {
            tt_entry
                .and_then(|entry| {
                    let se = entry.static_eval as i32;
                    if se < INF_EVAL { Some(se) } else { None }
                })
                .unwrap_or_else(|| self.evaluator.evaluate(board))
        };
        let static_eval = if in_check {
            VALUE_NONE
        } else if !self.stack[ply].excluded.is_null() {
            raw_static // already includes correction from parent
        } else {
            (raw_static + corr).clamp(-(MATE_SCORE - 1), MATE_SCORE - 1)
        };
        self.stack[ply].eval = static_eval;
        let improving = in_check
            || ply < 2
            || self.stack[ply - 2].eval == VALUE_NONE
            || static_eval > self.stack[ply - 2].eval;

        if !in_check && !pv_node {
            // Reverse futility pruning
            if depth <= 9 && static_eval - 120 * depth + if improving { 60 } else { 0 } >= beta {
                return static_eval;
            }
            // Razoring
            if depth <= 3 && static_eval + 300 * depth <= alpha {
                return self.quiescence(board, tt, alpha, beta, ply, should_stop);
            }
            // Null move pruning
            if !no_nmp
                && depth >= 3
                && board.has_non_pawn_material(board.side_to_move)
                && static_eval >= beta
            {
                let r = (4 + depth / 4 + ((static_eval - beta) / 200).min(3)).min(depth);
                board.make_null_move();
                self.stack[ply + 1] = SearchStack::default();
                let nmp_score = -self.negamax(
                    board,
                    tt,
                    (depth - r).max(0),
                    -beta,
                    -beta + 1,
                    ply + 1,
                    true,
                    should_stop,
                );
                board.unmake_null_move();
                if self.stop {
                    return 0;
                }
                if nmp_score >= beta {
                    // Don't return unverified mate claims
                    return nmp_score.min(MATE_SCORE - MAX_PLY as i32 - 1);
                }
            }

            // ProbCut: captures that are very likely to beat beta can be pruned
            if depth >= 5 && beta.abs() < MATE_SCORE - MAX_PLY as i32 {
                let pc_beta = (beta + 200).min(MATE_SCORE - MAX_PLY as i32 - 1);
                let mut caps = generate_captures(board);
                let caps_len = caps.len();
                let caps_slice = caps.as_mut_slice();
                let mut cap_scores = [0i32; MAX_MOVES];
                self.score_moves(
                    board,
                    caps_slice,
                    &mut cap_scores[..caps_len],
                    tt_move,
                    ply,
                    true,
                );
                for ci in 0..caps_len {
                    let mv = Self::next_move(caps_slice, &mut cap_scores[..caps_len], ci);
                    if board.see(mv) < pc_beta - static_eval {
                        continue;
                    }
                    board.make_move(mv);
                    // Shallow qsearch first to verify
                    let qs =
                        -self.quiescence(board, tt, -pc_beta, -pc_beta + 1, ply + 1, should_stop);
                    let pc_score = if !self.stop && qs >= pc_beta {
                        -self.negamax(
                            board,
                            tt,
                            depth - 4,
                            -pc_beta,
                            -pc_beta + 1,
                            ply + 1,
                            false,
                            should_stop,
                        )
                    } else {
                        qs
                    };
                    board.unmake_move(mv);
                    if self.stop {
                        return 0;
                    }
                    if pc_score >= pc_beta {
                        tt.store(
                            board.hash,
                            depth - 3,
                            TTFlag::Beta,
                            TranspositionTable::score_to_tt(pc_score, ply),
                            raw_static,
                            mv,
                        );
                        return pc_score;
                    }
                }
            }
        }

        // Internal iterative reduction
        if depth >= 4
            && (tt_move.is_null() || tt_entry.map_or(false, |e| (e.depth as i32) < depth - 3))
        {
            depth -= 1;
        }

        let mut moves = generate_legal_moves(board);
        if moves.is_empty() {
            return if in_check {
                -MATE_SCORE + ply as i32
            } else {
                0
            };
        }

        let moves_len = moves.len();
        let moves_slice = moves.as_mut_slice();
        let mut scores = [0i32; MAX_MOVES];
        self.score_moves(
            board,
            moves_slice,
            &mut scores[..moves_len],
            tt_move,
            ply,
            false,
        );

        let mut best_score = -INF_EVAL;
        let mut best_move = Move::NULL;
        let mut move_count = 0usize;
        let mut searched_any = false;
        let mut quiets = [Move::NULL; MAX_MOVES];
        let mut quiet_count = 0usize;
        let mut captures = [Move::NULL; MAX_MOVES];
        let mut capture_count = 0usize;

        for index in 0..moves_len {
            let mv = Self::next_move(moves_slice, &mut scores[..moves_len], index);
            if mv == self.stack[ply].excluded {
                continue;
            }

            let moving_piece = board.piece_type_at(mv.from_sq()).unwrap_or(Piece::Pawn);
            let is_capture = mv.is_capture();
            let is_quiet = !is_capture && !mv.is_promo();
            // Precompute victim (needed for capture history; must be before make_move)
            let captured_piece = if is_capture {
                victim_of(board, mv)
            } else {
                Piece::Pawn
            };
            let quiet_score = if is_quiet {
                self.quiet_history_score(board, mv, moving_piece, ply)
            } else {
                0
            };
            move_count += 1;

            if !pv_node && !in_check && best_score > -INF_EVAL / 2 {
                if is_quiet {
                    // LMP: skip late quiet moves (improving flag aware)
                    let lmp_threshold = if improving {
                        3 + depth as usize * depth as usize
                    } else {
                        2 + depth as usize * depth as usize / 2
                    };
                    if depth <= 8 && move_count > lmp_threshold {
                        continue;
                    }
                    // Futility pruning
                    if depth <= 6 && static_eval + 150 + 110 * depth <= alpha {
                        continue;
                    }
                    // History pruning: skip moves with very negative history
                    if depth <= 6 && quiet_score < -3500 * depth {
                        continue;
                    }
                } else {
                    // SEE pruning for bad captures
                    if depth <= 8 && board.see(mv) < -80 * depth {
                        continue;
                    }
                }
            }

            // -----------------------------------------------------------------------
            // Singular extensions (before making the move)
            // -----------------------------------------------------------------------
            let mut extension = 0i32;
            let is_tt_move = mv == tt_move && !tt_move.is_null();
            if is_tt_move
                && !in_check
                && depth >= 5
                && self.stack[ply].excluded.is_null()
                && tt_entry.map_or(false, |e| {
                    e.depth as i32 >= depth - 3
                        && e.flag() != TTFlag::Alpha
                        && e.flag() != TTFlag::None
                })
            {
                if let Some(entry) = tt_entry {
                    let tt_score = TranspositionTable::score_from_tt(entry.score as i32, ply);
                    // Only do singular test when tt_score is not a mate claim
                    if tt_score.abs() < MATE_SCORE - MAX_PLY as i32 {
                        let s_beta = tt_score - 2 * depth;
                        let s_depth = (depth - 1) / 2;
                        self.stack[ply].excluded = mv;
                        let s_score = self.negamax(
                            board,
                            tt,
                            s_depth,
                            s_beta - 1,
                            s_beta,
                            ply,
                            false,
                            should_stop,
                        );
                        self.stack[ply].excluded = Move::NULL;
                        if self.stop {
                            return 0;
                        }
                        if s_score < s_beta {
                            // Move is singular: extend
                            extension = if !pv_node && s_score + 20 < s_beta {
                                2 // double extension — very singular
                            } else {
                                1
                            };
                        } else if s_beta >= beta {
                            // Multicut: many moves beat beta, prune early
                            return s_beta;
                        } else if tt_score >= beta {
                            // TT move fails high but isn't uniquely good: negative extension
                            extension = -1;
                        }
                    }
                }
            }

            board.make_move(mv);
            let new_depth = (depth - 1 + extension).max(0);
            self.stack[ply + 1] = SearchStack {
                mv,
                excluded: Move::NULL,
                killers: self.stack[ply + 1].killers,
                eval: VALUE_NONE,
                moved_piece: moving_piece as u8,
            };

            let mut score;
            if pv_node && move_count == 1 {
                score = -self.negamax(
                    board,
                    tt,
                    new_depth,
                    -beta,
                    -alpha,
                    ply + 1,
                    false,
                    should_stop,
                );
            } else {
                let mut reduction = 0i32;
                if depth >= 2 && move_count > 2 {
                    if is_quiet && new_depth > 0 {
                        // LMR for quiet moves
                        reduction = self.lmr_table[depth.min(63) as usize][move_count.min(63)];
                        if !pv_node {
                            reduction += 1;
                        }
                        if !improving {
                            reduction += 1;
                        }
                        // History-based adjustment (basilisk: stat_score / 8192)
                        reduction -= quiet_score / 8192;
                        reduction = reduction.clamp(0, new_depth - 1);
                    } else if !is_quiet && !mv.is_promo() && new_depth > 0 {
                        // Bad captures: base LMR halved (basilisk: (reduction-1)/2)
                        let base = self.lmr_table[depth.min(63) as usize][move_count.min(63)];
                        reduction = ((base - 1) / 2).clamp(0, new_depth - 1);
                    }
                }

                score = -self.negamax(
                    board,
                    tt,
                    (new_depth - reduction).max(0),
                    -alpha - 1,
                    -alpha,
                    ply + 1,
                    false,
                    should_stop,
                );
                if self.stop {
                    board.unmake_move(mv);
                    return 0;
                }
                if reduction > 0 && score > alpha {
                    score = -self.negamax(
                        board,
                        tt,
                        new_depth,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        false,
                        should_stop,
                    );
                }
                if pv_node && score > alpha && score < beta {
                    score = -self.negamax(
                        board,
                        tt,
                        new_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        false,
                        should_stop,
                    );
                }
            }
            board.unmake_move(mv);
            if self.stop {
                return 0;
            }

            searched_any = true;
            if is_quiet && quiet_count < MAX_MOVES {
                quiets[quiet_count] = mv;
                quiet_count += 1;
            } else if is_capture && capture_count < MAX_MOVES {
                captures[capture_count] = mv;
                capture_count += 1;
            }

            if score > best_score {
                best_score = score;
                best_move = mv;
                if ply == 0 {
                    self.stack[0].mv = mv;
                }
            }
            if score > alpha {
                alpha = score;
                if alpha >= beta {
                    let bonus = (depth * depth).min(2048);
                    if is_quiet {
                        self.update_killers(ply, mv);
                        self.update_countermove(ply, mv);
                        self.update_quiet_history(board, ply, mv, moving_piece, bonus);
                        for quiet in quiets[..quiet_count]
                            .iter()
                            .copied()
                            .filter(|quiet| *quiet != mv)
                        {
                            let piece = board.piece_type_at(quiet.from_sq()).unwrap_or(Piece::Pawn);
                            self.update_quiet_history(board, ply, quiet, piece, -bonus);
                        }
                    } else {
                        self.update_capture_history(mv, moving_piece, captured_piece, bonus);
                        for capture in captures[..capture_count]
                            .iter()
                            .copied()
                            .filter(|capture| *capture != mv)
                        {
                            let attacker = board
                                .piece_type_at(capture.from_sq())
                                .unwrap_or(Piece::Pawn);
                            let victim = victim_of(board, capture);
                            self.update_capture_history(capture, attacker, victim, -bonus);
                        }
                    }
                    tt.store(
                        board.hash,
                        depth,
                        TTFlag::Beta,
                        TranspositionTable::score_to_tt(score, ply),
                        tt_static_eval(static_eval),
                        mv,
                    );
                    return score;
                }
            }
        }

        if !searched_any {
            return alpha;
        }

        if !best_move.is_null() {
            let bonus = (depth * depth).min(2048);
            if !best_move.is_capture() && !best_move.is_promo() {
                let piece = board
                    .piece_type_at(best_move.from_sq())
                    .unwrap_or(Piece::Pawn);
                self.update_quiet_history(board, ply, best_move, piece, bonus);
                for quiet in quiets[..quiet_count]
                    .iter()
                    .copied()
                    .filter(|quiet| *quiet != best_move)
                {
                    let piece = board.piece_type_at(quiet.from_sq()).unwrap_or(Piece::Pawn);
                    self.update_quiet_history(board, ply, quiet, piece, -bonus);
                }
            }
        }

        if !in_check
            && self.stack[ply].excluded.is_null()
            && static_eval != VALUE_NONE
            && best_score.abs() < MATE_SCORE - MAX_PLY as i32
            && (best_score >= beta || best_score > original_alpha)
        {
            let delta = (best_score - static_eval).clamp(-256, 256) * depth.max(1);
            let corr_entry = &mut self.corr_hist[board.side_to_move as usize][corr_index];
            *corr_entry = (*corr_entry + delta / 16).clamp(-CORR_HIST_MAX, CORR_HIST_MAX);
        }

        let flag = if best_score > original_alpha {
            TTFlag::Exact
        } else {
            TTFlag::Alpha
        };
        tt.store(
            board.hash,
            depth,
            flag,
            TranspositionTable::score_to_tt(best_score, ply),
            tt_static_eval(static_eval),
            best_move,
        );
        best_score
    }

    fn quiescence<F>(
        &mut self,
        board: &mut Board,
        tt: &mut TranspositionTable,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        should_stop: &mut F,
    ) -> i32
    where
        F: FnMut() -> bool,
    {
        self.nodes += 1;
        self.seldepth = self.seldepth.max(ply);
        if self.check_stop(should_stop) {
            return 0;
        }
        if ply >= MAX_PLY - 1 || board.is_draw() {
            return self.evaluator.evaluate(board);
        }

        let in_check = board.is_in_check();

        // TT probe in qsearch
        let tt_entry = tt.probe(board.hash);
        let tt_move_qs = tt_entry.map(|e| Move(e.mv)).unwrap_or(Move::NULL);
        if let Some(entry) = tt_entry {
            let tt_score = TranspositionTable::score_from_tt(entry.score as i32, ply);
            match entry.flag() {
                TTFlag::Exact => return tt_score,
                TTFlag::Alpha if tt_score <= alpha => return tt_score,
                TTFlag::Beta if tt_score >= beta => return tt_score,
                _ => {}
            }
        }

        let stand_pat = if in_check {
            VALUE_NONE
        } else {
            let raw = tt_entry
                .and_then(|e| {
                    if (e.static_eval as i32) < INF_EVAL {
                        Some(e.static_eval as i32)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| self.evaluator.evaluate(board));
            let corr_index = (board.pawn_hash() as usize) & 16383;
            let corr = self.corr_hist[board.side_to_move as usize][corr_index] / 16;
            (raw + corr).clamp(-(MATE_SCORE - 1), MATE_SCORE - 1)
        };
        if !in_check {
            if stand_pat >= beta {
                tt.store(
                    board.hash,
                    0,
                    TTFlag::Beta,
                    TranspositionTable::score_to_tt(stand_pat, ply),
                    tt_static_eval(stand_pat),
                    Move::NULL,
                );
                return stand_pat;
            }
            // Delta pruning
            if stand_pat < alpha - 1100 {
                return alpha;
            }
            alpha = alpha.max(stand_pat);
        }

        let mut moves = if in_check {
            generate_legal_moves(board)
        } else {
            generate_captures(board)
        };
        if moves.is_empty() {
            return if in_check {
                -MATE_SCORE + ply as i32
            } else {
                alpha
            };
        }

        let moves_len = moves.len();
        let moves_slice = moves.as_mut_slice();
        let mut scores = [0i32; MAX_MOVES];
        self.score_moves(
            board,
            moves_slice,
            &mut scores[..moves_len],
            tt_move_qs,
            ply,
            true,
        );

        let orig_alpha = alpha;
        let mut best = if in_check {
            -INF_EVAL
        } else {
            stand_pat.max(alpha)
        };
        let mut best_move = Move::NULL;
        for index in 0..moves_len {
            let mv = Self::next_move(moves_slice, &mut scores[..moves_len], index);
            if !in_check {
                let delta = if mv.is_capture() {
                    piece_value(victim_of(board, mv))
                } else if mv.is_promo() {
                    piece_value(mv.promo_piece()) - piece_value(Piece::Pawn)
                } else {
                    0
                };
                if stand_pat != VALUE_NONE && stand_pat + delta + 200 < alpha {
                    continue;
                }
                if mv.is_capture() && board.see(mv) < 0 {
                    continue;
                }
            }

            let moving_piece = board.piece_type_at(mv.from_sq()).unwrap_or(Piece::Pawn);
            board.make_move(mv);
            self.stack[ply + 1] = SearchStack {
                mv,
                excluded: Move::NULL,
                killers: self.stack[ply + 1].killers,
                eval: VALUE_NONE,
                moved_piece: moving_piece as u8,
            };
            let score = -self.quiescence(board, tt, -beta, -alpha, ply + 1, should_stop);
            board.unmake_move(mv);
            if self.stop {
                return 0;
            }
            if score > best {
                best = score;
                best_move = mv;
                if score > alpha {
                    alpha = score;
                    if alpha >= beta {
                        // Store TT beta cutoff
                        tt.store(
                            board.hash,
                            0,
                            TTFlag::Beta,
                            TranspositionTable::score_to_tt(score, ply),
                            tt_static_eval(stand_pat),
                            mv,
                        );
                        return best;
                    }
                }
            }
        }
        // Store TT result
        let flag = if alpha > orig_alpha {
            TTFlag::Exact
        } else {
            TTFlag::Alpha
        };
        tt.store(
            board.hash,
            0,
            flag,
            TranspositionTable::score_to_tt(best, ply),
            tt_static_eval(stand_pat),
            best_move,
        );
        best
    }

    fn score_moves(
        &self,
        board: &Board,
        moves: &[Move],
        scores: &mut [i32],
        tt_move: Move,
        ply: usize,
        qsearch: bool,
    ) {
        for (index, &mv) in moves.iter().enumerate() {
            scores[index] = if mv == tt_move {
                10_000_000
            } else if mv.is_capture() {
                let attacker = board.piece_type_at(mv.from_sq()).unwrap_or(Piece::Pawn);
                let victim = victim_of(board, mv);
                let hist =
                    self.cap_hist[attacker as usize][mv.to_sq().index()][victim as usize] as i32;
                let see = board.see(mv);
                if see >= 0 {
                    6_000_000 + piece_value(victim) * 16 - piece_value(attacker) + hist
                } else {
                    2_000_000 + see
                }
            } else if mv.is_promo() {
                if mv.promo_piece() == Piece::Queen {
                    5_500_000
                } else {
                    -100_000 + piece_value(mv.promo_piece())
                }
            } else if !qsearch && mv == self.stack[ply].killers[0] {
                4_000_000
            } else if !qsearch && mv == self.stack[ply].killers[1] {
                3_900_000
            } else if !qsearch
                && ply > 0
                && mv
                    == self.countermove[self.stack[ply - 1].mv.from_sq().index()]
                        [self.stack[ply - 1].mv.to_sq().index()]
            {
                3_800_000
            } else if qsearch {
                board.see(mv)
            } else {
                let piece = board.piece_type_at(mv.from_sq()).unwrap_or(Piece::Pawn);
                self.quiet_history_score(board, mv, piece, ply)
            };
        }
    }

    fn next_move(moves: &mut [Move], scores: &mut [i32], start: usize) -> Move {
        let mut best = start;
        for index in (start + 1)..moves.len() {
            if scores[index] > scores[best] {
                best = index;
            }
        }
        moves.swap(start, best);
        scores.swap(start, best);
        moves[start]
    }

    fn quiet_history_score(&self, board: &Board, mv: Move, piece: Piece, ply: usize) -> i32 {
        let from = mv.from_sq().index();
        let to = mv.to_sq().index();
        let mut score = self.main_hist[board.side_to_move as usize * 64 + from][to] as i32;
        if ply > 0 {
            let prev = &self.stack[ply - 1];
            if prev.moved_piece != 255 {
                score += self.cont_hist1[prev.moved_piece as usize][prev.mv.to_sq().index()]
                    [piece as usize][to] as i32;
            }
        }
        if ply > 1 {
            let prev = &self.stack[ply - 2];
            if prev.moved_piece != 255 {
                score += self.cont_hist2[prev.moved_piece as usize][prev.mv.to_sq().index()]
                    [piece as usize][to] as i32;
            }
        }
        score
    }

    fn update_quiet_history(
        &mut self,
        board: &Board,
        ply: usize,
        mv: Move,
        piece: Piece,
        bonus: i32,
    ) {
        let from = mv.from_sq().index();
        let to = mv.to_sq().index();
        update_history_entry(
            &mut self.main_hist[board.side_to_move as usize * 64 + from][to],
            bonus,
        );
        if ply > 0 {
            let prev = &self.stack[ply - 1];
            if prev.moved_piece != 255 {
                update_history_entry(
                    &mut self.cont_hist1[prev.moved_piece as usize][prev.mv.to_sq().index()]
                        [piece as usize][to],
                    bonus,
                );
            }
        }
        if ply > 1 {
            let prev = &self.stack[ply - 2];
            if prev.moved_piece != 255 {
                update_history_entry(
                    &mut self.cont_hist2[prev.moved_piece as usize][prev.mv.to_sq().index()]
                        [piece as usize][to],
                    bonus,
                );
            }
        }
    }

    fn update_capture_history(&mut self, mv: Move, attacker: Piece, victim: Piece, bonus: i32) {
        update_history_entry(
            &mut self.cap_hist[attacker as usize][mv.to_sq().index()][victim as usize],
            bonus,
        );
    }

    fn update_killers(&mut self, ply: usize, mv: Move) {
        if mv != self.stack[ply].killers[0] {
            self.stack[ply].killers[1] = self.stack[ply].killers[0];
            self.stack[ply].killers[0] = mv;
        }
    }

    fn update_countermove(&mut self, ply: usize, mv: Move) {
        if ply > 0 {
            let prev = self.stack[ply - 1].mv;
            if !prev.is_null() {
                self.countermove[prev.from_sq().index()][prev.to_sq().index()] = mv;
            }
        }
    }

    fn extract_pv(&self, board: &Board, tt: &TranspositionTable, depth: usize) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut temp = board.clone();
        for _ in 0..depth {
            let Some(entry) = tt.probe(temp.hash) else {
                break;
            };
            let mv = Move(entry.mv);
            if mv.is_null() {
                break;
            }
            let legal = generate_legal_moves(&temp);
            if !legal.as_slice().contains(&mv) {
                break;
            }
            pv.push(mv);
            temp.make_move(mv);
            if temp.is_draw() {
                break;
            }
        }
        pv
    }

    fn check_stop<F>(&mut self, should_stop: &mut F) -> bool
    where
        F: FnMut() -> bool,
    {
        if self.stop {
            return true;
        }
        if (self.nodes & 1023) == 0 {
            if self.elapsed_ms() >= self.hard_limit || should_stop() {
                self.stop = true;
            }
        }
        self.stop
    }

    fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    fn dynamic_soft_limit(&self, stability: u32) -> u64 {
        if self.soft_limit == u64::MAX {
            u64::MAX
        } else {
            self.soft_limit
                .saturating_mul(100 - 6 * stability.min(6) as u64)
                / 100
        }
    }
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}

fn update_history_entry(entry: &mut i16, bonus: i32) {
    let bonus = bonus.clamp(-HISTORY_MAX, HISTORY_MAX);
    let value = *entry as i32;
    let updated = value + bonus - value * bonus.abs() / HISTORY_MAX;
    *entry = updated.clamp(-HISTORY_MAX, HISTORY_MAX) as i16;
}

fn tt_static_eval(eval: i32) -> i32 {
    if eval == VALUE_NONE {
        VALUE_NONE
    } else {
        eval.clamp(-(MATE_SCORE - 1), MATE_SCORE - 1)
    }
}

fn victim_of(board: &Board, mv: Move) -> Piece {
    if mv.is_en_passant() {
        Piece::Pawn
    } else {
        board.piece_type_at(mv.to_sq()).unwrap_or(Piece::Pawn)
    }
}

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight | Piece::Bishop => 300,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiescence_in_check_returns_real_score() {
        let mut board = Board::from_fen("4k3/8/8/8/4Q3/8/8/4K3 b - - 0 1").unwrap();
        assert!(board.is_in_check());

        let mut tt = TranspositionTable::new(1);
        let mut searcher = Searcher::new();
        let mut should_stop = || false;
        let score = searcher.quiescence(
            &mut board,
            &mut tt,
            -INF_EVAL + 1,
            INF_EVAL - 1,
            0,
            &mut should_stop,
        );

        assert_ne!(score, VALUE_NONE);
        assert!(score.abs() < INF_EVAL);
        assert_eq!(tt.probe(board.hash).unwrap().static_eval as i32, VALUE_NONE);
    }
}
