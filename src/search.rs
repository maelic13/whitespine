use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    mpsc,
};
use std::time::Instant;

use crate::board::{Board, Color, GameResult, Move, Piece};
use crate::eval::{Evaluator, INF_SCORE, MATE_SCORE, VALUE_NONE, piece_value};
use crate::move_ordering::{
    BadCaptureList, CAP_HISTORY_MAX, CONT_SIZE, CORR_SIZE, HISTORY_MAX, ScoredMoveList, cont_index,
    diversify_root_scores, history_bonus, pick_next, update_hist_entry,
};
use crate::search_options::{EngineOptions, MAX_THREADS, SearchLimits, SearchOptions};
use crate::search_threads::{STOP_NONE, STOP_QUIT, STOP_SEARCH, WorkerJob, WorkerPool};
use crate::time_manager::{RuntimeLimits, compute_runtime_limits};
use crate::tt::{Bound, TranspositionTable, score_from_tt};

const MAX_DEPTH: usize = 100;
const MAX_PLY: usize = 128;
const MAX_QPLY: usize = 10;
const MIN_PARALLEL_DEPTH: usize = 4;
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SearchEvent {
    None,
    Stop,
    Quit,
    PonderHit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SearchExit {
    Stop,
    Quit,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub bestmove: Move,
    pub pondermove: Move,
    pub score: i32,
    pub depth: usize,
    pub nodes: u64,
    pub elapsed_ms: u128,
    pub exit: SearchExit,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum TtWriteMode {
    Main,
    Helper,
}

pub struct Searcher {
    tt: TranspositionTable,
    hash_mb: usize,
    worker_pool: WorkerPool,
    evaluator: Evaluator,
    nodes: u64,
    seldepth: usize,
    stopped: bool,
    quit: bool,
    pondering: bool,
    start: Instant,
    limits: RuntimeLimits,
    pv_table: [[Move; MAX_PLY]; MAX_PLY],
    pv_len: [usize; MAX_PLY],
    stack_moves: [Move; MAX_PLY],
    stack_pieces: [Piece; MAX_PLY],
    killers: [[Move; 2]; MAX_PLY],
    main_history: Box<[[[i16; 64]; 64]; 2]>,
    cap_history: Box<[[[i16; 6]; 64]; 6]>,
    cont_history_1: Vec<i16>,
    cont_history_2: Vec<i16>,
    correction_history: Box<[[i16; CORR_SIZE]; 2]>,
    countermove: Box<[[Move; 64]; 64]>,
    root_move_offset: usize,
    tt_write_mode: TtWriteMode,
}

impl Default for Searcher {
    fn default() -> Self {
        Self {
            tt: TranspositionTable::default(),
            hash_mb: 64,
            worker_pool: WorkerPool::default(),
            evaluator: Evaluator::default(),
            nodes: 0,
            seldepth: 0,
            stopped: false,
            quit: false,
            pondering: false,
            start: Instant::now(),
            limits: RuntimeLimits {
                depth: MAX_DEPTH,
                nodes: 0,
                soft_ms: f64::INFINITY,
                hard_ms: f64::INFINITY,
            },
            pv_table: [[Move::NULL; MAX_PLY]; MAX_PLY],
            pv_len: [0; MAX_PLY],
            stack_moves: [Move::NULL; MAX_PLY],
            stack_pieces: [Piece::Pawn; MAX_PLY],
            killers: [[Move::NULL; 2]; MAX_PLY],
            main_history: Box::new([[[0; 64]; 64]; 2]),
            cap_history: Box::new([[[0; 6]; 64]; 6]),
            cont_history_1: vec![0; CONT_SIZE],
            cont_history_2: vec![0; CONT_SIZE],
            correction_history: Box::new([[0; CORR_SIZE]; 2]),
            countermove: Box::new([[Move::NULL; 64]; 64]),
            root_move_offset: 0,
            tt_write_mode: TtWriteMode::Main,
        }
    }
}

impl Searcher {
    pub(crate) fn worker_default() -> Self {
        Self {
            worker_pool: WorkerPool::default(),
            ..Self::default()
        }
    }

    pub(crate) fn reset_worker_state_for_new_game(&mut self) {
        self.clear_history();
        self.evaluator.clear_pawn_table();
    }

    pub(crate) fn run_worker_job<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        job: WorkerJob,
        poll: &mut P,
    ) -> SearchResult {
        self.tt = job.tt;
        self.hash_mb = job.hash_mb;
        self.root_move_offset = job.root_move_offset;
        self.tt_write_mode = TtWriteMode::Helper;
        self.search_worker(
            job.root,
            job.limits,
            job.engine_options,
            job.root_moves.as_ref(),
            poll,
        )
    }

    pub fn configure(&mut self, options: &SearchOptions) {
        self.configure_engine(options.engine);
    }

    fn configure_engine(&mut self, options: EngineOptions) {
        if options.hash_mb != self.hash_mb {
            if self.tt.resize(options.hash_mb) {
                self.hash_mb = options.hash_mb;
            } else {
                println!(
                    "info string Unable to allocate Hash value {}; keeping {} MiB.",
                    options.hash_mb, self.hash_mb
                );
            }
        }
        if options.clear_hash {
            self.tt.clear();
        }
        self.worker_pool
            .set_helper_count(options.threads.saturating_sub(1));
    }

    pub fn new_game(&mut self) {
        self.tt.clear();
        self.clear_history();
        self.evaluator.clear_pawn_table();
        self.worker_pool.new_game();
    }

    pub fn clear_history(&mut self) {
        self.main_history = Box::new([[[0; 64]; 64]; 2]);
        self.cap_history = Box::new([[[0; 6]; 64]; 6]);
        self.cont_history_1.fill(0);
        self.cont_history_2.fill(0);
        self.correction_history = Box::new([[0; CORR_SIZE]; 2]);
        self.countermove = Box::new([[Move::NULL; 64]; 64]);
        self.killers = [[Move::NULL; 2]; MAX_PLY];
    }

    pub fn hashfull(&self) -> usize {
        self.tt.hashfull()
    }

    pub fn search(
        &mut self,
        root: Board,
        options: &SearchOptions,
        emit_info: bool,
        mut poll: impl FnMut() -> SearchEvent,
    ) -> SearchResult {
        self.search_impl::<true, _>(root, options.limits, options.engine, emit_info, &mut poll)
    }

    fn search_impl<const ALLOW_PARALLEL: bool, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        limits: SearchLimits,
        engine_options: EngineOptions,
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        if ALLOW_PARALLEL && engine_options.threads <= 1 && !self.tt.ensure_local(self.hash_mb) {
            println!(
                "info string Unable to restore local transposition table at {} MiB.",
                self.hash_mb
            );
        }
        self.root_move_offset = 0;
        self.tt_write_mode = TtWriteMode::Main;
        self.reset_search_state(limits, engine_options, root.side_to_move(), true, true);

        let board = root;
        let legal_moves = board.generate_legal_movelist();
        if legal_moves.is_empty() {
            return self.no_legal_moves_result(&board);
        }

        if ALLOW_PARALLEL {
            let threads = engine_options
                .threads
                .clamp(1, MAX_THREADS)
                .min(legal_moves.len().max(1));
            if threads > 1
                && limits.nodes == 0
                && self.limits.depth.min(MAX_DEPTH - 1) >= MIN_PARALLEL_DEPTH
            {
                return self.search_parallel(
                    board,
                    legal_moves.as_slice(),
                    limits,
                    engine_options,
                    threads,
                    emit_info,
                    poll,
                );
            }
        }

        self.search_root(board, legal_moves.as_slice(), emit_info, poll)
    }

    fn reset_search_state(
        &mut self,
        limits: SearchLimits,
        engine_options: EngineOptions,
        side_to_move: Color,
        age_tt: bool,
        age_history: bool,
    ) {
        self.start = Instant::now();
        self.nodes = 0;
        self.seldepth = 0;
        self.stopped = false;
        self.quit = false;
        self.pondering = limits.ponder;
        self.limits = compute_runtime_limits(limits, engine_options, side_to_move, MAX_DEPTH);
        if age_tt {
            self.tt.new_search();
        }
        if age_history {
            self.age_history();
        }
        self.pv_table = [[Move::NULL; MAX_PLY]; MAX_PLY];
        self.pv_len = [0; MAX_PLY];
        self.stack_moves = [Move::NULL; MAX_PLY];
        self.stack_pieces = [Piece::Pawn; MAX_PLY];
    }

    fn no_legal_moves_result(&mut self, board: &Board) -> SearchResult {
        let result = self.result_for_no_legal_moves(board);
        SearchResult {
            bestmove: Move::NULL,
            pondermove: Move::NULL,
            score: self
                .evaluator
                .evaluate_result(result, board.side_to_move(), 0),
            depth: 0,
            nodes: 0,
            elapsed_ms: self.start.elapsed().as_millis(),
            exit: SearchExit::Stop,
        }
    }

    fn search_root<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        mut board: Board,
        legal_moves: &[Move],
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        let mut bestmove = legal_moves[0];
        let mut pondermove = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut completed_depth = 0;
        let max_depth = self.limits.depth.min(MAX_DEPTH - 1);
        let mut stable_best_depths = 0usize;

        for depth in 1..=max_depth {
            let previous_bestmove = bestmove;
            let use_aspiration = depth >= 4 && best_score.abs() < MATE_SCORE - MAX_PLY as i32;
            let mut alpha = if use_aspiration {
                best_score - 25
            } else {
                -INF_SCORE
            };
            let mut beta = if use_aspiration {
                best_score + 25
            } else {
                INF_SCORE
            };

            loop {
                let score = self.negamax(
                    &mut board,
                    depth as i32,
                    alpha,
                    beta,
                    0,
                    true,
                    true,
                    Move::NULL,
                    poll,
                );
                if self.stopped || self.quit {
                    break;
                }
                if score <= alpha {
                    alpha = (alpha - 75).max(-INF_SCORE);
                    beta = (alpha + beta) / 2;
                    continue;
                }
                if score >= beta {
                    beta = (beta + 75).min(INF_SCORE);
                    continue;
                }
                best_score = score;
                completed_depth = depth;
                if self.pv_len[0] > 0 {
                    bestmove = self.pv_table[0][0];
                    pondermove = if self.pv_len[0] > 1 {
                        self.pv_table[0][1]
                    } else {
                        Move::NULL
                    };
                    if bestmove == previous_bestmove {
                        stable_best_depths += 1;
                    } else {
                        stable_best_depths = 0;
                    }
                }
                break;
            }

            if self.stopped || self.quit {
                break;
            }

            if emit_info {
                self.send_info(depth, best_score);
            }

            if !self.pondering {
                let elapsed_ms = self.elapsed_ms();
                if elapsed_ms >= self.limits.hard_ms
                    || (elapsed_ms >= self.limits.soft_ms
                        && (self.limits.soft_ms >= self.limits.hard_ms || stable_best_depths > 0))
                {
                    break;
                }
            }
        }

        SearchResult {
            bestmove,
            pondermove,
            score: best_score,
            depth: completed_depth,
            nodes: self.nodes,
            elapsed_ms: self.start.elapsed().as_millis(),
            exit: if self.quit {
                SearchExit::Quit
            } else {
                SearchExit::Stop
            },
        }
    }

    fn search_worker<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        limits: SearchLimits,
        engine_options: EngineOptions,
        legal_moves: &[Move],
        poll: &mut P,
    ) -> SearchResult {
        self.reset_search_state(limits, engine_options, root.side_to_move(), false, true);
        self.search_root(root, legal_moves, false, poll)
    }

    #[cold]
    #[inline(never)]
    fn search_parallel<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        root_moves: &[Move],
        limits: SearchLimits,
        engine_options: EngineOptions,
        threads: usize,
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        self.tt.make_shared();
        let stop_state = Arc::new(AtomicU8::new(STOP_NONE));
        let helper_count = threads.min(root_moves.len()).saturating_sub(1);
        let root_len = root_moves.len();
        let mut worker_engine_options = engine_options;
        worker_engine_options.threads = 1;
        self.worker_pool.set_helper_count(helper_count);
        let root_moves_shared: Arc<[Move]> = root_moves.to_vec().into();

        let (result_tx, result_rx) = mpsc::channel();
        let mut launched_helpers = 0usize;
        for index in 0..helper_count {
            let offset = ((index + 1) * root_len / threads).max(1) % root_len;
            let job = WorkerJob {
                root: root.clone(),
                root_moves: Arc::clone(&root_moves_shared),
                limits,
                engine_options: worker_engine_options,
                tt: self.tt.clone(),
                hash_mb: self.hash_mb,
                root_move_offset: offset,
                stop_state: Arc::clone(&stop_state),
                result_tx: result_tx.clone(),
            };
            if self.worker_pool.send_search(index, job) {
                launched_helpers += 1;
            }
        }
        drop(result_tx);

        self.root_move_offset = 0;
        let mut main_poll = || match stop_state.load(Ordering::Relaxed) {
            STOP_QUIT => SearchEvent::Quit,
            STOP_SEARCH => SearchEvent::Stop,
            _ => match poll() {
                SearchEvent::Quit => {
                    stop_state.store(STOP_QUIT, Ordering::Relaxed);
                    SearchEvent::Quit
                }
                SearchEvent::Stop => {
                    stop_state.store(STOP_SEARCH, Ordering::Relaxed);
                    SearchEvent::Stop
                }
                SearchEvent::PonderHit => SearchEvent::PonderHit,
                SearchEvent::None => SearchEvent::None,
            },
        };
        let main_result = self.search_root(root, root_moves, emit_info, &mut main_poll);
        stop_state
            .compare_exchange(STOP_NONE, STOP_SEARCH, Ordering::Relaxed, Ordering::Relaxed)
            .ok();

        let mut helper_results = Vec::with_capacity(launched_helpers + 1);
        helper_results.push(main_result);
        for _ in 0..launched_helpers {
            if let Ok(result) = result_rx.recv() {
                helper_results.push(result);
            }
        }
        self.root_move_offset = 0;

        let mut total_nodes = 0u64;
        let mut quit = false;
        for result in &helper_results {
            total_nodes = total_nodes.saturating_add(result.nodes);
            quit |= result.exit == SearchExit::Quit;
        }
        let mut best =
            select_parallel_result(&helper_results, root_moves).unwrap_or(SearchResult {
                bestmove: root_moves[0],
                pondermove: Move::NULL,
                score: -INF_SCORE,
                depth: 0,
                nodes: 0,
                elapsed_ms: self.start.elapsed().as_millis(),
                exit: SearchExit::Stop,
            });
        self.nodes = total_nodes;
        self.quit = quit;
        self.stopped = true;
        best.nodes = total_nodes;
        best.elapsed_ms = self.start.elapsed().as_millis();
        best.exit = if quit {
            SearchExit::Quit
        } else {
            SearchExit::Stop
        };
        best
    }

    fn negamax<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        mut depth: i32,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        is_pv: bool,
        allow_null: bool,
        excluded: Move,
        poll: &mut P,
    ) -> i32 {
        if self.check_stop(poll) {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return self.corrected_eval(board);
        }
        self.pv_len[ply] = ply;
        self.seldepth = self.seldepth.max(ply);

        if ply > 0 && board.can_declare_draw_in_search() {
            return 0;
        }

        let in_check = board.is_in_check();
        if in_check {
            depth += 1;
        }

        let mate_alpha = -MATE_SCORE + ply as i32;
        let mate_beta = MATE_SCORE - ply as i32 - 1;
        alpha = alpha.max(mate_alpha);
        let beta = beta.min(mate_beta);
        if alpha >= beta {
            return alpha;
        }

        if depth <= 0 {
            return self.quiescence(board, alpha, beta, ply, 0, poll);
        }

        let original_alpha = alpha;
        let hash = board.hash;
        let tt_entry = self.tt.probe(hash);
        let tt_move = tt_entry
            .and_then(|entry| entry.best_move())
            .unwrap_or(Move::NULL);
        let tt_score = tt_entry
            .map(|entry| score_from_tt(entry.score as i32, ply))
            .unwrap_or(VALUE_NONE);
        let tt_depth = tt_entry.map(|entry| entry.depth as i32).unwrap_or(-1);
        let tt_bound = tt_entry.and_then(|entry| entry.bound());
        if !is_pv
            && excluded.is_null()
            && let Some(entry) = tt_entry
            && entry.depth as i32 >= depth
            && let Some(bound) = entry.bound()
        {
            let score = score_from_tt(entry.score as i32, ply);
            match bound {
                Bound::Exact => return score,
                Bound::Lower if score >= beta => return score,
                Bound::Upper if score <= alpha => return score,
                _ => {}
            }
        }

        if !is_pv && excluded.is_null() && depth >= 4 && tt_move.is_null() {
            depth -= 1;
        }

        let static_eval = if in_check {
            VALUE_NONE
        } else if let Some(entry) = tt_entry {
            if entry.static_eval as i32 != VALUE_NONE {
                entry.static_eval as i32
            } else {
                self.corrected_eval(board)
            }
        } else {
            self.corrected_eval(board)
        };
        if !is_pv && !in_check && excluded.is_null() {
            if depth <= 7 && static_eval - 80 * depth >= beta {
                return static_eval;
            }
            if depth <= 3 && static_eval + 150 * depth < alpha {
                return self.quiescence(board, alpha, beta, ply, 0, poll);
            }
            if allow_null
                && depth >= 3
                && static_eval >= beta
                && board.has_non_pawn_material(board.side_to_move())
            {
                let reduction = 3 + depth / 4 + ((static_eval - beta) / 200).clamp(0, 3);
                board.make_null_move();
                let score = -self.negamax(
                    board,
                    depth - reduction,
                    -beta,
                    -beta + 1,
                    ply + 1,
                    false,
                    false,
                    Move::NULL,
                    poll,
                );
                board.unmake_null_move();
                if self.stopped || self.quit {
                    return 0;
                }
                if score >= beta {
                    return beta;
                }
            }

            if depth >= 4 {
                let probcut_beta = beta + 160;
                let captures = board.generate_legal_captures();
                let mut scored = self.score_moves(board, captures.as_slice(), tt_move, ply);
                for index in 0..scored.len().min(8) {
                    let picked = pick_next(scored.as_mut_slice(), index);
                    let mv = picked.mv;
                    if picked.see < 0 {
                        continue;
                    }
                    self.stack_moves[ply] = mv;
                    board.make_move_unchecked(mv);
                    let score =
                        -self.quiescence(board, -probcut_beta, -probcut_beta + 1, ply + 1, 0, poll);
                    let score = if score >= probcut_beta {
                        -self.negamax(
                            board,
                            depth - 4,
                            -probcut_beta,
                            -probcut_beta + 1,
                            ply + 1,
                            false,
                            false,
                            Move::NULL,
                            poll,
                        )
                    } else {
                        score
                    };
                    board.unmake_move(mv);
                    self.stack_moves[ply] = Move::NULL;
                    if self.stopped || self.quit {
                        return 0;
                    }
                    if score >= probcut_beta {
                        return beta;
                    }
                }
            }
        }

        let legal_moves = board.generate_legal_movelist();
        if legal_moves.is_empty() {
            return if in_check {
                -MATE_SCORE + ply as i32
            } else {
                0
            };
        }

        let mut scored = self.score_moves(board, legal_moves.as_slice(), tt_move, ply);
        if ply == 0 && self.root_move_offset > 0 && scored.len() > 1 {
            let offset = self.root_move_offset % scored.len();
            diversify_root_scores(scored.as_mut_slice(), offset);
        }
        let mut best_move = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut searched = 0usize;
        let mut quiets = crate::board::MoveList::new();
        let mut bad_caps = BadCaptureList::new();
        let previous_move = if ply > 0 {
            self.stack_moves[ply - 1]
        } else {
            Move::NULL
        };

        for index in 0..scored.len() {
            let picked = pick_next(scored.as_mut_slice(), index);
            let mv = picked.mv;
            if mv == excluded {
                continue;
            }
            let is_capture = mv.is_capture();
            let is_quiet = board.is_quiet_move(mv);
            let see = if is_capture { picked.see as i32 } else { 0 };
            let moving_piece = board.moving_piece(mv);
            let captured_piece = board.captured_piece(mv);
            let quiet_hist = if is_quiet {
                self.quiet_history_score(board, board.side_to_move(), mv, ply)
            } else {
                0
            };

            if !is_pv && !in_check && searched > 0 {
                if is_quiet {
                    if depth <= 3 && static_eval + 100 * depth <= alpha {
                        continue;
                    }
                    if depth <= 8 && searched > late_move_prune_count(depth) {
                        continue;
                    }
                    if depth <= 4 && quiet_hist < -8_000 {
                        continue;
                    }
                } else if depth <= 7 && see < -80 * depth {
                    continue;
                }
            }

            let child_is_pv = is_pv && searched == 0;
            let mut extension = 0;
            if ply > 0
                && mv == tt_move
                && excluded.is_null()
                && depth >= 5
                && tt_depth >= depth - 3
                && matches!(tt_bound, Some(Bound::Lower | Bound::Exact))
                && tt_score.abs() < MATE_SCORE - MAX_PLY as i32
            {
                let singular_beta = tt_score - 2 * depth;
                let singular_depth = (depth - 1) / 2;
                let singular_score = self.negamax(
                    board,
                    singular_depth,
                    singular_beta - 1,
                    singular_beta,
                    ply,
                    false,
                    false,
                    mv,
                    poll,
                );
                if self.stopped || self.quit {
                    return 0;
                }
                if singular_score < singular_beta {
                    extension = if !is_pv && singular_score < singular_beta - 20 {
                        2
                    } else {
                        1
                    };
                } else if singular_beta >= beta {
                    return singular_beta;
                } else if tt_score >= beta {
                    extension = -1;
                }
            }

            self.stack_moves[ply] = mv;
            self.stack_pieces[ply] = moving_piece;
            board.make_move_unchecked(mv);
            let new_depth = depth - 1 + extension;
            let mut score;

            if searched == 0 {
                score = -self.negamax(
                    board,
                    new_depth,
                    -beta,
                    -alpha,
                    ply + 1,
                    child_is_pv,
                    true,
                    Move::NULL,
                    poll,
                );
            } else {
                let reducible =
                    depth >= 3 && searched >= 2 && (is_quiet || see < 0) && !mv.is_promo();
                if reducible {
                    let hist = quiet_hist;
                    let mut reduction = lmr_reduction(depth, searched);
                    if is_pv {
                        reduction -= 1;
                    }
                    if hist > 6_000 {
                        reduction -= 1;
                    } else if hist < -6_000 {
                        reduction += 1;
                    }
                    reduction = reduction.clamp(1, new_depth.max(1));
                    score = -self.negamax(
                        board,
                        new_depth - reduction,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        false,
                        true,
                        Move::NULL,
                        poll,
                    );
                    if score > alpha {
                        score = -self.negamax(
                            board,
                            new_depth,
                            -alpha - 1,
                            -alpha,
                            ply + 1,
                            false,
                            true,
                            Move::NULL,
                            poll,
                        );
                    }
                } else {
                    score = -self.negamax(
                        board,
                        new_depth,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        false,
                        true,
                        Move::NULL,
                        poll,
                    );
                }
                if score > alpha && score < beta {
                    score = -self.negamax(
                        board,
                        new_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        true,
                        true,
                        Move::NULL,
                        poll,
                    );
                }
            }
            board.unmake_move(mv);
            self.stack_moves[ply] = Move::NULL;

            if self.stopped || self.quit {
                return 0;
            }

            searched += 1;
            if score > best_score {
                best_score = score;
                best_move = mv;
            }
            if score > alpha {
                alpha = score;
                self.pv_table[ply][ply] = mv;
                let child_len = self.pv_len[ply + 1].max(ply + 1);
                for next_ply in ply + 1..child_len {
                    self.pv_table[ply][next_ply] = self.pv_table[ply + 1][next_ply];
                }
                self.pv_len[ply] = child_len;

                if score >= beta {
                    if excluded.is_null() {
                        if !is_capture {
                            self.update_cutoff_tables(
                                board.side_to_move(),
                                mv,
                                moving_piece,
                                previous_move,
                                ply,
                                depth,
                                quiets.as_slice(),
                                &bad_caps,
                            );
                        } else {
                            self.update_capture_history(
                                moving_piece,
                                mv.to_sq().index(),
                                captured_piece,
                                depth * depth,
                            );
                        }
                        self.store_tt(hash, depth, beta, Bound::Lower, mv, ply, static_eval);
                    }
                    return beta;
                }
            }

            if is_quiet {
                quiets.push(mv);
            } else if is_capture && see < 0 {
                bad_caps.push(moving_piece, mv.to_sq().index(), captured_piece);
            }
        }

        let bound = if best_score > original_alpha {
            Bound::Exact
        } else {
            Bound::Upper
        };
        if bound == Bound::Exact
            && excluded.is_null()
            && !in_check
            && static_eval != VALUE_NONE
            && best_score.abs() < MATE_SCORE - MAX_PLY as i32
        {
            self.update_correction(
                board.side_to_move(),
                board.pawn_key(),
                best_score - static_eval,
                depth,
            );
        }
        if excluded.is_null() {
            self.store_tt(hash, depth, alpha, bound, best_move, ply, static_eval);
        }
        alpha
    }

    fn quiescence<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        qply: usize,
        poll: &mut P,
    ) -> i32 {
        if self.check_stop(poll) {
            return 0;
        }
        self.pv_len[ply] = ply;
        self.seldepth = self.seldepth.max(ply);

        if board.can_declare_draw_in_search() {
            return 0;
        }

        let in_check = board.is_in_check();
        let tt_entry = self.tt.probe(board.hash);
        let tt_move = tt_entry
            .and_then(|entry| entry.best_move())
            .unwrap_or(Move::NULL);
        if let Some(entry) = tt_entry
            && entry.depth >= 0
            && let Some(bound) = entry.bound()
        {
            let score = score_from_tt(entry.score as i32, ply);
            match bound {
                Bound::Exact => return score,
                Bound::Lower if score >= beta => return score,
                Bound::Upper if score <= alpha => return score,
                _ => {}
            }
        }

        if !in_check {
            let stand_pat = if let Some(entry) = tt_entry {
                if entry.static_eval as i32 != VALUE_NONE {
                    entry.static_eval as i32
                } else {
                    self.corrected_eval(board)
                }
            } else {
                self.corrected_eval(board)
            };
            if stand_pat >= beta {
                return beta;
            }
            if qply >= MAX_QPLY {
                return stand_pat.max(alpha);
            }
            if stand_pat > alpha {
                alpha = stand_pat;
            }
            if board.occupied_count() > 8 && stand_pat + piece_value(Piece::Queen) + 200 < alpha {
                return alpha;
            }
        }

        let moves = if in_check {
            board.generate_legal_movelist()
        } else if qply < 2 {
            self.quiescence_moves(board)
        } else {
            board.generate_legal_captures()
        };

        if in_check && moves.is_empty() {
            return -MATE_SCORE + ply as i32;
        }

        let mut scored = self.score_moves(board, moves.as_slice(), tt_move, ply);
        for index in 0..scored.len() {
            let picked = pick_next(scored.as_mut_slice(), index);
            let mv = picked.mv;
            if !in_check && picked.see < -50 {
                continue;
            }
            let moving_piece = board.moving_piece(mv);
            self.stack_moves[ply] = mv;
            self.stack_pieces[ply] = moving_piece;
            board.make_move_unchecked(mv);
            let score = -self.quiescence(board, -beta, -alpha, ply + 1, qply + 1, poll);
            board.unmake_move(mv);
            self.stack_moves[ply] = Move::NULL;
            if self.stopped || self.quit {
                return 0;
            }
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
                self.pv_table[ply][ply] = mv;
                let child_len = self.pv_len[ply + 1].max(ply + 1);
                for next_ply in ply + 1..child_len {
                    self.pv_table[ply][next_ply] = self.pv_table[ply + 1][next_ply];
                }
                self.pv_len[ply] = child_len;
            }
        }
        alpha
    }

    fn quiescence_moves(&mut self, board: &mut Board) -> crate::board::MoveList {
        let mut moves = board.generate_legal_captures();

        for &mv in board.generate_legal_quiets().as_slice() {
            if mv.is_promo() || board.gives_check(mv) {
                moves.push(mv);
            }
        }

        moves
    }

    fn score_moves(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
        ply: usize,
    ) -> ScoredMoveList {
        let mut scored = ScoredMoveList::new();
        let previous = if ply > 0 {
            self.stack_moves[ply - 1]
        } else {
            Move::NULL
        };
        let counter = if !previous.is_null() {
            self.countermove[previous.from_sq().index()][previous.to_sq().index()]
        } else {
            Move::NULL
        };

        for &mv in moves {
            let mut see = 0;
            let score = if mv == tt_move {
                30_000_000
            } else if mv.is_capture() {
                let attacker = board.piece_on(mv.from_sq()).unwrap_or(Piece::Pawn);
                let victim = board.captured_piece(mv).unwrap_or(Piece::Pawn);
                see = board.see(mv);
                let hist =
                    self.cap_history[attacker as usize][mv.to_sq().index()][victim as usize] as i32;
                if see >= 0 {
                    20_000_000 + 32 * see + 10 * piece_value(victim) - piece_value(attacker) + hist
                } else {
                    8_000_000 + see + hist
                }
            } else if mv.is_promo() {
                18_000_000 + piece_value(mv.promo_piece())
            } else if mv == self.killers[ply][0] {
                16_000_000
            } else if mv == self.killers[ply][1] {
                15_900_000
            } else if mv == counter {
                15_800_000
            } else {
                self.quiet_history_score(board, board.side_to_move(), mv, ply)
            };
            scored.push(mv, score, see);
        }
        scored
    }

    fn quiet_history_score(&self, board: &Board, color: Color, mv: Move, ply: usize) -> i32 {
        let from = mv.from_sq().index();
        let to = mv.to_sq().index();
        let main = self.main_history[color as usize][from][to] as i32;
        let piece = board.moving_piece(mv) as usize;
        main + self.cont_score(ply, piece, to)
    }

    fn cont_score(&self, ply: usize, piece: usize, to: usize) -> i32 {
        let mut score = 0;
        if ply >= 1 {
            let prev = self.stack_moves[ply - 1];
            if !prev.is_null() {
                score += self.cont_history_1[cont_index(
                    self.stack_pieces[ply - 1] as usize,
                    prev.to_sq().index(),
                    piece,
                    to,
                )] as i32;
            }
        }
        if ply >= 2 {
            let prev = self.stack_moves[ply - 2];
            if !prev.is_null() {
                score += self.cont_history_2[cont_index(
                    self.stack_pieces[ply - 2] as usize,
                    prev.to_sq().index(),
                    piece,
                    to,
                )] as i32;
            }
        }
        score
    }

    fn update_cutoff_tables(
        &mut self,
        color: Color,
        best: Move,
        best_piece: Piece,
        previous: Move,
        ply: usize,
        depth: i32,
        quiets: &[Move],
        bad_caps: &BadCaptureList,
    ) {
        if self.killers[ply][0] != best {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = best;
        }

        let bonus = history_bonus(depth);
        self.update_quiet_history(color, best, bonus);
        for &quiet in quiets {
            self.update_quiet_history(color, quiet, -bonus);
        }
        for bad_cap in bad_caps.as_slice() {
            self.update_capture_history(bad_cap.attacker, bad_cap.to, bad_cap.captured, -bonus);
        }

        if !previous.is_null() {
            self.countermove[previous.from_sq().index()][previous.to_sq().index()] = best;
        }

        let piece = best_piece as usize;
        let to = best.to_sq().index();
        if ply >= 1 {
            let prev = self.stack_moves[ply - 1];
            if !prev.is_null() {
                update_hist_entry(
                    &mut self.cont_history_1[cont_index(
                        self.stack_pieces[ply - 1] as usize,
                        prev.to_sq().index(),
                        piece,
                        to,
                    )],
                    bonus,
                    HISTORY_MAX,
                );
            }
        }
        if ply >= 2 {
            let prev = self.stack_moves[ply - 2];
            if !prev.is_null() {
                update_hist_entry(
                    &mut self.cont_history_2[cont_index(
                        self.stack_pieces[ply - 2] as usize,
                        prev.to_sq().index(),
                        piece,
                        to,
                    )],
                    bonus,
                    HISTORY_MAX,
                );
            }
        }
    }

    fn update_quiet_history(&mut self, color: Color, mv: Move, bonus: i32) {
        update_hist_entry(
            &mut self.main_history[color as usize][mv.from_sq().index()][mv.to_sq().index()],
            bonus,
            HISTORY_MAX,
        );
    }

    fn update_capture_history(
        &mut self,
        attacker: Piece,
        to: usize,
        captured: Option<Piece>,
        bonus: i32,
    ) {
        if let Some(captured) = captured {
            update_hist_entry(
                &mut self.cap_history[attacker as usize][to][captured as usize],
                bonus,
                CAP_HISTORY_MAX,
            );
        }
    }

    fn age_history(&mut self) {
        for color in self.main_history.iter_mut() {
            for from in color.iter_mut() {
                for value in from.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for attacker in self.cap_history.iter_mut() {
            for to in attacker.iter_mut() {
                for value in to.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for value in &mut self.cont_history_1 {
            *value /= 2;
        }
        for value in &mut self.cont_history_2 {
            *value /= 2;
        }
        for color in self.correction_history.iter_mut() {
            for value in color.iter_mut() {
                *value /= 2;
            }
        }
    }

    fn corrected_eval(&mut self, board: &Board) -> i32 {
        let raw = self.evaluator.evaluate(board);
        raw + self.correction_value(board.side_to_move(), board.pawn_key())
    }

    fn correction_value(&self, color: Color, pawn_key: u64) -> i32 {
        self.correction_history[color as usize][pawn_key as usize & (CORR_SIZE - 1)] as i32 / 32
    }

    fn update_correction(&mut self, color: Color, pawn_key: u64, diff: i32, depth: i32) {
        let scaled = (diff * depth.max(1)).clamp(-1024, 1024);
        update_hist_entry(
            &mut self.correction_history[color as usize][pawn_key as usize & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
    }

    fn store_tt(
        &mut self,
        key: u64,
        depth: i32,
        score: i32,
        bound: Bound,
        mv: Move,
        ply: usize,
        static_eval: i32,
    ) {
        if self.tt_write_mode == TtWriteMode::Helper {
            let min_depth = match bound {
                Bound::Exact => 3,
                Bound::Lower => 5,
                Bound::Upper => 7,
            };
            if depth < min_depth {
                return;
            }
        }
        self.tt
            .store(key, depth, score, bound, mv, ply, static_eval);
    }

    fn check_stop<P: FnMut() -> SearchEvent + ?Sized>(&mut self, poll: &mut P) -> bool {
        self.nodes += 1;
        if self.limits.nodes > 0 && self.nodes >= self.limits.nodes {
            self.stopped = true;
            return true;
        }
        if self.nodes & 2047 == 0 {
            match poll() {
                SearchEvent::Quit => {
                    self.quit = true;
                    self.stopped = true;
                }
                SearchEvent::Stop => {
                    self.stopped = true;
                }
                SearchEvent::PonderHit => {
                    self.pondering = false;
                    self.start = Instant::now();
                }
                SearchEvent::None => {}
            }
            if !self.pondering && self.elapsed_ms() >= self.limits.hard_ms {
                self.stopped = true;
            }
        }
        self.stopped
    }

    fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    fn send_info(&self, depth: usize, score: i32) {
        let elapsed_ms = self.start.elapsed().as_millis();
        let nps = if elapsed_ms > 0 {
            self.nodes as u128 * 1000 / elapsed_ms
        } else {
            self.nodes as u128
        };
        let pv = self.pv_table[0][..self.pv_len[0].min(MAX_PLY)]
            .iter()
            .copied()
            .filter(|mv| !mv.is_null())
            .map(|mv| mv.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "info depth {} seldepth {} score {} nodes {} nps {} hashfull {} time {} pv {}",
            depth,
            self.seldepth,
            format_score(score),
            self.nodes,
            nps,
            self.hashfull(),
            elapsed_ms,
            pv
        );
    }

    fn result_for_no_legal_moves(&self, board: &Board) -> GameResult {
        if board.is_in_check() {
            match board.side_to_move() {
                Color::White => GameResult::BlackCheckmates,
                Color::Black => GameResult::WhiteCheckmates,
            }
        } else {
            GameResult::Stalemate
        }
    }
}

fn lmr_reduction(depth: i32, move_index: usize) -> i32 {
    if depth < 3 || move_index < 2 {
        return 0;
    }
    let d = depth as f64;
    let m = move_index as f64;
    (0.75 + d.ln() * m.ln() / 2.25) as i32
}

fn late_move_prune_count(depth: i32) -> usize {
    (3 + depth * depth / 2) as usize
}

fn select_parallel_result(results: &[SearchResult], root_moves: &[Move]) -> Option<SearchResult> {
    let max_depth = results
        .iter()
        .filter(|result| is_root_result(result, root_moves))
        .map(|result| result.depth)
        .max()?;

    let mut votes: Vec<(Move, usize, bool, i32, usize)> = Vec::new();
    for (index, result) in results.iter().enumerate() {
        if result.depth != max_depth || !is_root_result(result, root_moves) {
            continue;
        }
        if let Some(vote) = votes
            .iter_mut()
            .find(|(mv, _, _, _, _)| *mv == result.bestmove)
        {
            vote.1 += 1;
            vote.2 |= index == 0;
            if result.score > vote.3 {
                vote.3 = result.score;
                vote.4 = index;
            }
        } else {
            votes.push((result.bestmove, 1, index == 0, result.score, index));
        }
    }

    votes
        .into_iter()
        .max_by_key(|(_, count, main_vote, score, _)| (*count, *main_vote, *score))
        .and_then(|(_, _, _, _, index)| results.get(index).cloned())
        .or_else(|| {
            results
                .iter()
                .filter(|result| is_root_result(result, root_moves))
                .max_by_key(|result| (result.depth, result.score))
                .cloned()
        })
}

fn is_root_result(result: &SearchResult, root_moves: &[Move]) -> bool {
    result.depth > 0 && root_moves.iter().any(|&mv| mv == result.bestmove)
}

fn format_score(score: i32) -> String {
    if score >= MATE_SCORE - MAX_PLY as i32 {
        format!("mate {}", (MATE_SCORE - score + 1) / 2)
    } else if score <= -MATE_SCORE + MAX_PLY as i32 {
        format!("mate -{}", (MATE_SCORE + score + 1) / 2)
    } else {
        format!("cp {score}")
    }
}
