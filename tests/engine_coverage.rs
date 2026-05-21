use whitespine::board::{Board, Color, GameResult, Move, Piece, Square};
use whitespine::engine_command::EngineCommand;
use whitespine::eval::{Evaluator, MATE_SCORE, piece_value};
use whitespine::search::{SearchEvent, SearchExit, Searcher};
use whitespine::search_options::SearchOptions;
use whitespine::tt::{Bound, TranspositionTable, score_from_tt, score_to_tt};

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn piece_at(board: &Board, square: Square) -> Option<(Color, Piece)> {
    board.piece_at(square)
}

#[test]
fn search_options_parse_startpos_moves_and_go_limits() {
    let mut options = SearchOptions::default();

    options.set_position(&args(&["startpos", "moves", "e2e4", "e7e5", "g1f3"]));

    assert_eq!(options.position.board.side_to_move(), Color::Black);
    assert_eq!(
        piece_at(&options.position.board, Square::E4),
        Some((Color::White, Piece::Pawn))
    );
    assert_eq!(
        piece_at(&options.position.board, Square::E5),
        Some((Color::Black, Piece::Pawn))
    );
    assert_eq!(
        piece_at(&options.position.board, Square::F3),
        Some((Color::White, Piece::Knight))
    );

    options.set_search_parameters(&args(&[
        "wtime",
        "10000",
        "btime",
        "9000",
        "winc",
        "100",
        "binc",
        "200",
        "movestogo",
        "20",
        "nodes",
        "12345",
        "ponder",
        "depth",
        "7",
    ]));

    assert_eq!(options.limits.white_time, 10_000);
    assert_eq!(options.limits.black_time, 9_000);
    assert_eq!(options.limits.white_increment, 100);
    assert_eq!(options.limits.black_increment, 200);
    assert_eq!(options.limits.movestogo, 20);
    assert_eq!(options.limits.nodes, 12_345);
    assert_eq!(options.limits.depth, 7.0);
    assert!(options.limits.ponder);
}

#[test]
fn search_options_setoption_and_reset_cover_engine_configuration() {
    let mut options = SearchOptions::default();

    options.set_option(&args(&["name", "Hash", "value", "256"]));
    options.set_option(&args(&["name", "Move", "Overhead", "value", "25"]));
    options.set_option(&args(&["name", "Threads", "value", "99"]));
    options.set_option(&args(&["name", "Clear", "Hash"]));

    assert_eq!(options.engine.hash_mb, 256);
    assert_eq!(options.engine.move_overhead, 25.0);
    assert_eq!(options.engine.threads, 99);
    assert!(options.engine.clear_hash);

    options.set_option(&args(&["name", "Threads", "value", "9999"]));
    assert_eq!(options.engine.threads, 1024);

    options.set_search_parameters(&args(&[
        "depth", "4", "nodes", "99", "movetime", "500", "ponder",
    ]));
    options.reset();

    assert_eq!(options.limits.depth, f64::INFINITY);
    assert_eq!(options.limits.nodes, 0);
    assert_eq!(options.limits.move_time, 0);
    assert!(!options.limits.ponder);
    assert_eq!(options.engine.hash_mb, 256);
    assert_eq!(options.engine.move_overhead, 25.0);

    let names = SearchOptions::get_uci_options().join("\n");
    assert!(names.contains("option name Hash"));
    assert!(names.contains("option name Move Overhead"));
    assert!(names.contains("option name Threads type spin default 1 min 1 max 1024"));
    assert!(names.contains("option name Clear Hash"));
}

#[test]
fn search_options_reject_illegal_position_move_without_losing_current_board() {
    let mut options = SearchOptions::default();

    options.set_position(&args(&["startpos", "moves", "e2e4"]));
    let expected = options.position.board.clone();

    options.set_position(&args(&["startpos", "moves", "e2e5"]));

    assert_eq!(options.position.board.hash, expected.hash);
    assert_eq!(options.position.board.to_fen(), expected.to_fen());
}

#[test]
fn engine_command_constructors_set_expected_flags() {
    let options = SearchOptions::default();

    let go = EngineCommand::go(options, 11);
    assert!(!go.stop);
    assert!(!go.quit);
    assert!(go.bench_depth.is_none());
    assert!(go.configure.is_none());
    assert!(!go.new_game);
    assert!(!go.ponderhit);
    assert_eq!(go.epoch, 11);

    let stop = EngineCommand::stop(12);
    assert!(stop.stop);
    assert!(!stop.quit);
    assert_eq!(stop.epoch, 12);

    let quit = EngineCommand::quit(13);
    assert!(quit.quit);
    assert!(quit.stop);
    assert_eq!(quit.epoch, 13);

    let bench = EngineCommand::bench(7, SearchOptions::default(), 14);
    assert_eq!(bench.bench_depth, Some(7));
    assert_eq!(bench.epoch, 14);

    let configure = EngineCommand::configure(SearchOptions::default());
    assert!(configure.configure.is_some());

    let new_game = EngineCommand::new_game();
    assert!(new_game.new_game);

    let ponderhit = EngineCommand::ponderhit();
    assert!(ponderhit.ponderhit);
}

#[test]
fn transposition_table_store_probe_replace_clear_and_mate_scores() {
    let mut table = TranspositionTable::new(1);
    let key = 0x1234_0000_0000_0000;
    let best = Move::from_uci("e2e4").expect("valid UCI move");

    table.store(key, 5, 123, Bound::Exact, best, 0, 42);
    let entry = table.probe(key).expect("entry must be stored");
    assert_eq!(entry.score, 123);
    assert_eq!(entry.static_eval, 42);
    assert_eq!(entry.depth, 5);
    assert_eq!(entry.bound(), Some(Bound::Exact));
    assert_eq!(entry.best_move(), Some(best));

    table.store(0x5678_0000_0000_0001, 1, 1, Bound::Exact, best, 0, 0);
    assert!(table.hashfull() > 0);

    assert!(!table.resize(usize::MAX));
    assert!(
        table.probe(key).is_some(),
        "failed resize must keep the current table"
    );

    table.make_shared();
    assert!(
        table.probe(key).is_none(),
        "shared TT should not import local key16-only entries"
    );
    table.store(key, 5, 123, Bound::Exact, best, 0, 42);
    let shared_entry = table
        .probe(key)
        .expect("entry must be stored in shared table");
    assert_eq!(shared_entry.score, 123);
    assert_eq!(shared_entry.bound(), Some(Bound::Exact));
    assert_eq!(shared_entry.best_move(), Some(best));
    assert!(
        table.probe(key ^ 0x0000_8000_0000_0000).is_none(),
        "shared TT must validate the full key"
    );

    table.store(key, 4, 90, Bound::Upper, Move::NULL, 0, 11);
    let replaced = table.probe(key).expect("entry must remain present");
    assert_eq!(replaced.bound(), Some(Bound::Upper));
    assert_eq!(replaced.best_move(), Some(best));

    let mate_in_three = MATE_SCORE - 3;
    let tt_score = score_to_tt(mate_in_three, 7);
    assert_eq!(score_from_tt(tt_score, 7), mate_in_three);
    let mated_in_three = -MATE_SCORE + 3;
    let tt_score = score_to_tt(mated_in_three, 7);
    assert_eq!(score_from_tt(tt_score, 7), mated_in_three);

    table.clear();
    assert!(table.probe(key).is_none());
    assert_eq!(table.hashfull(), 0);
}

#[test]
fn evaluator_scores_material_from_side_to_move_perspective() {
    let mut evaluator = Evaluator::default();
    let white_to_move = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 w - - 0 1").expect("valid FEN");
    let black_to_move = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 b - - 0 1").expect("valid FEN");

    assert!(evaluator.evaluate(&white_to_move) > piece_value(Piece::Queen) - 100);
    assert!(evaluator.evaluate(&black_to_move) < -piece_value(Piece::Queen) + 100);
}

#[test]
fn evaluator_rewards_advanced_protected_passers_over_back_rank_pawns() {
    let mut evaluator = Evaluator::default();
    let advanced_connected =
        Board::from_fen("4k3/8/4P3/3P4/8/8/8/4K3 w - - 0 1").expect("valid FEN");
    let undeveloped = Board::from_fen("4k3/8/8/8/8/3P4/4P3/4K3 w - - 0 1").expect("valid FEN");

    assert!(evaluator.evaluate(&advanced_connected) > evaluator.evaluate(&undeveloped));
}

#[test]
fn evaluator_scales_known_drawish_minor_endgames() {
    let mut evaluator = Evaluator::default();
    let two_knights_vs_bare_king =
        Board::from_fen("7k/8/8/8/8/8/8/NN2K3 w - - 0 1").expect("valid FEN");

    assert_eq!(evaluator.evaluate(&two_knights_vs_bare_king), 0);
}

#[test]
fn evaluator_reports_terminal_results_with_distance_to_mate() {
    let evaluator = Evaluator::default();

    assert_eq!(
        evaluator.evaluate_result(GameResult::WhiteCheckmates, Color::White, 3),
        MATE_SCORE - 3
    );
    assert_eq!(
        evaluator.evaluate_result(GameResult::WhiteCheckmates, Color::Black, 3),
        -MATE_SCORE + 3
    );
    assert_eq!(
        evaluator.evaluate_result(GameResult::BlackCheckmates, Color::Black, 11),
        MATE_SCORE - 11
    );
    assert_eq!(
        evaluator.evaluate_result(GameResult::Draw, Color::White, 99),
        0
    );
}

#[test]
fn search_returns_null_move_for_stalemate() {
    let board = Board::from_fen("4k3/4P3/4K3/8/8/8/8/8 b - - 0 1").expect("valid FEN");
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = 4.0;

    let result = searcher.search(board, &options, false, || SearchEvent::None);

    assert_eq!(result.bestmove, Move::NULL);
    assert_eq!(result.pondermove, Move::NULL);
    assert_eq!(result.depth, 0);
    assert_eq!(result.score, 0);
    assert_eq!(result.exit, SearchExit::Stop);
}

#[test]
fn search_respects_node_limit() {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = 99.0;
    options.limits.nodes = 512;

    let result = searcher.search(options.position.board.clone(), &options, false, || {
        SearchEvent::None
    });

    assert_eq!(result.exit, SearchExit::Stop);
    assert!(result.nodes >= 512, "nodes: {}", result.nodes);
    assert!(result.nodes <= 2_048, "nodes: {}", result.nodes);
    assert!(result.depth < 99);
}

#[test]
fn threaded_search_keeps_node_limit_on_main_search() {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = 99.0;
    options.limits.nodes = 512;
    options.engine.threads = 8;

    let result = searcher.search(options.position.board.clone(), &options, false, || {
        SearchEvent::None
    });

    assert_eq!(result.exit, SearchExit::Stop);
    assert!(result.nodes >= 512, "nodes: {}", result.nodes);
    assert!(
        result.nodes <= 2_048,
        "threaded node-limited search should not multiply the limit: {}",
        result.nodes
    );
    assert!(result.depth < 99);
}

#[test]
fn search_quit_event_exits_search() {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = 99.0;

    let mut polls = 0;
    let result = searcher.search(options.position.board.clone(), &options, false, || {
        polls += 1;
        SearchEvent::Quit
    });

    assert_eq!(result.exit, SearchExit::Quit);
    assert!(polls > 0);
    assert!(result.nodes >= 512, "nodes: {}", result.nodes);
    assert!(result.depth < 99);
}
