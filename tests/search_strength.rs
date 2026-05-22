use whitespine::board::{Board, Move};
use whitespine::eval::MATE_SCORE;
use whitespine::search::{SearchEvent, SearchResult, Searcher};
use whitespine::search_options::SearchOptions;

#[test]
fn search_finds_fools_mate_in_one() {
    let board =
        Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2").unwrap();

    let result = search_at_depth(board, 1);
    assert_eq!(result.to_string(), "d8h4");
}

#[test]
fn threaded_search_finds_fools_mate_in_one() {
    let board =
        Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2").unwrap();

    let result = search_at_depth_with_threads(board, 1, 2);
    assert_eq!(result.to_string(), "d8h4");
}

#[test]
fn search_continues_to_resolve_shorter_mate() {
    let board = Board::from_fen("4K3/2Q5/6k1/8/8/8/8/8 w - - 0 1").unwrap();

    let result = search_result_at_depth_with_threads(board, 18, 1);

    assert_eq!(result.depth, 18);
    assert!(result.score >= MATE_SCORE - 9);
    assert!(mate_in_from_score(result.score) <= 5);
    assert!(!result.bestmove.is_null());
}

#[test]
fn search_prefers_winning_hanging_queen() {
    let board = Board::from_fen("4k3/8/8/8/3q4/2N1B3/8/4K3 w - - 0 1").unwrap();

    let result = search_at_depth(board, 2);
    assert_eq!(result.to_string(), "e3d4");
}

#[test]
fn fixed_depth_single_thread_search_is_repeatable() {
    let board = Board::default();
    let first = search_result_at_depth_with_threads(board.clone(), 4, 1);
    let second = search_result_at_depth_with_threads(board, 4, 1);

    assert_eq!(first.bestmove, second.bestmove);
    assert_eq!(first.score, second.score);
    assert_eq!(first.depth, second.depth);
    assert_eq!(first.nodes, second.nodes);
}

#[test]
fn searcher_handles_thread_count_changes() {
    let board =
        Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2").unwrap();
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = 4.0;

    options.position.board = board.clone();
    options.engine.threads = 4;
    let threaded = searcher.search(board.clone(), &options, false, || SearchEvent::None);

    options.position.board = board.clone();
    options.engine.threads = 1;
    let single = searcher.search(board, &options, false, || SearchEvent::None);

    assert_eq!(threaded.bestmove.to_string(), "d8h4");
    assert_eq!(single.bestmove.to_string(), "d8h4");
}

fn search_at_depth(board: Board, depth: usize) -> Move {
    search_at_depth_with_threads(board, depth, 1)
}

fn search_result_at_depth_with_threads(board: Board, depth: usize, threads: usize) -> SearchResult {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.position.board = board.clone();
    options.limits.depth = depth as f64;
    options.engine.threads = threads;
    searcher.search(board, &options, false, || SearchEvent::None)
}

fn search_at_depth_with_threads(board: Board, depth: usize, threads: usize) -> Move {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.position.board = board.clone();
    options.limits.depth = depth as f64;
    options.engine.threads = threads;
    let result = searcher.search(board, &options, false, || SearchEvent::None);
    result.bestmove
}

fn mate_in_from_score(score: i32) -> i32 {
    (MATE_SCORE - score.abs() + 1) / 2
}
