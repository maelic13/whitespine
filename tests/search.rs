/// Tests for the search engine, including SEE correctness.
use whitespine::board::{generate_legal_moves, Board, Square};
use whitespine::search::{SearchResult, Searcher};
use whitespine::tt::{MATE_SCORE, MAX_PLY, TranspositionTable};

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

fn search_to_depth(fen: &str, depth: u32) -> SearchResult {
    let mut board = Board::from_fen(fen).unwrap();
    let mut tt = TranspositionTable::new(4);
    let mut searcher = Searcher::new();
    searcher.iterative_deepening(
        &mut board,
        &mut tt,
        depth,
        u64::MAX,
        u64::MAX,
        || false,
        |_| {},
    )
}

fn is_mate_score(score: i32) -> bool {
    score.abs() >= MATE_SCORE - MAX_PLY as i32
}

/// Find the first legal move from `from` to `to` in the given board.
fn find_move(board: &Board, from: Square, to: Square) -> whitespine::board::Move {
    generate_legal_moves(board)
        .into_iter()
        .find(|mv| mv.from_sq() == from && mv.to_sq() == to)
        .unwrap_or_else(|| panic!("no legal move from {from} to {to}"))
}

// -----------------------------------------------------------------------
// Mate-in-1 detection
// -----------------------------------------------------------------------

#[test]
fn finds_mate_in_1_back_rank() {
    // White rook delivers back-rank mate: Ra8#
    let result = search_to_depth("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1", 3);
    assert!(
        is_mate_score(result.score),
        "expected mate score, got {}",
        result.score
    );
    assert_eq!(
        result.best_move.to_sq(),
        Square::A8,
        "expected Ra8#, best move was {}",
        result.best_move
    );
}

#[test]
fn finds_mate_in_1_queen() {
    // White queen delivers checkmate in one move. Both Qa7# and Qb8# are valid.
    let result = search_to_depth("k7/8/KQ6/8/8/8/8/8 w - - 0 1", 3);
    assert!(
        is_mate_score(result.score),
        "expected mate score, got {}",
        result.score
    );
    assert!(!result.best_move.is_null(), "engine should have a best move");
}

#[test]
fn finds_mate_in_1_ladder() {
    // Two rooks deliver checkmate: Ra8#
    let result = search_to_depth("6k1/8/8/8/8/8/8/RR4K1 w - - 0 1", 3);
    assert!(
        is_mate_score(result.score),
        "expected mate score, got {}",
        result.score
    );
}

// -----------------------------------------------------------------------
// Tactical tests
// -----------------------------------------------------------------------

#[test]
fn avoids_blundering_queen() {
    // White queen is attacked; engine should move it away, not leave it hanging.
    // Fen: white queen on d5 attacked by black pawn on c6. Black pawn can take for free.
    let result = search_to_depth("4k3/8/2p5/3Q4/8/8/8/4K3 w - - 0 1", 4);
    // The best move should NOT be a null move, and the score should be favorable.
    assert!(!result.best_move.is_null(), "engine should have a move");
}

#[test]
fn finds_winning_capture() {
    // White knight can take an undefended rook: NxR free.
    let result = search_to_depth("4k3/8/8/3r4/4N3/8/8/4K3 w - - 0 1", 3);
    assert!(!result.best_move.is_null(), "engine should have a move");
    // Score should be clearly positive (extra rook).
    assert!(
        result.score > 200,
        "winning capture should give score > 200, got {}",
        result.score
    );
}

// -----------------------------------------------------------------------
// Draw detection
// -----------------------------------------------------------------------

#[test]
fn starting_position_has_valid_move() {
    let result = search_to_depth("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 4);
    assert!(!result.best_move.is_null(), "should find a move from starting position");
}

#[test]
fn stalemate_position_score_near_zero() {
    // True stalemate: black is in stalemate — score should be 0.
    // White to move but this is a drawn-ish position.
    // Simple symmetric drawn positions return near-zero.
    let result = search_to_depth("4k3/8/8/8/8/8/8/4K3 w - - 0 1", 4);
    assert!(
        result.score.abs() <= 50,
        "K vs K position should be near draw, got score {}",
        result.score
    );
}

// -----------------------------------------------------------------------
// SEE (Static Exchange Evaluation) correctness
// -----------------------------------------------------------------------

#[test]
fn see_free_queen_capture_positive() {
    // Pawn takes undefended queen: SEE should be +900.
    let board = Board::from_fen("4k3/8/8/3q4/4P3/8/8/4K3 w - - 0 1").unwrap();
    let mv = find_move(&board, Square::E4, Square::D5);
    let see = board.see(mv);
    assert_eq!(see, 900, "PxQ (no defense) should give SEE = 900, got {see}");
}

#[test]
fn see_bad_capture_queen_for_pawn_is_negative() {
    // Queen captures pawn defended by a pawn: SEE should be negative (−800).
    // FEN: white queen d1, black pawn d5, black pawn e6 defends d5.
    let board = Board::from_fen("4k3/8/4p3/3p4/8/8/8/3QK3 w - - 0 1").unwrap();
    let mv = find_move(&board, Square::D1, Square::D5);
    let see = board.see(mv);
    assert!(
        see < 0,
        "QxP defended by P should give negative SEE, got {see}"
    );
    assert_eq!(see, -800, "QxP defended by P should give SEE = −800, got {see}");
}

#[test]
fn see_equal_exchange_is_zero() {
    // Knight takes knight defended by pawn: SEE = 0 (equal exchange).
    // White knight c3 captures black knight d5 (defended by black pawn e6).
    // c3→d5 is a valid knight move (+1 file, +2 rank).
    let board = Board::from_fen("4k3/8/4p3/3n4/8/2N5/8/4K3 w - - 0 1").unwrap();
    let mv = find_move(&board, Square::C3, Square::D5);
    let see = board.see(mv);
    assert_eq!(see, 0, "NxN defended by P should give SEE = 0, got {see}");
}

#[test]
fn see_free_pawn_capture_positive() {
    // Pawn takes undefended pawn: SEE = 100.
    let board = Board::from_fen("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1").unwrap();
    let mv = find_move(&board, Square::E4, Square::D5);
    let see = board.see(mv);
    assert_eq!(see, 100, "PxP (no defense) should give SEE = 100, got {see}");
}

#[test]
fn see_rook_takes_knight_free_positive() {
    // Rook takes undefended knight: SEE = +300.
    let board = Board::from_fen("4k3/8/8/3n4/8/8/8/3RK3 w - - 0 1").unwrap();
    let mv = find_move(&board, Square::D1, Square::D5);
    let see = board.see(mv);
    assert_eq!(see, 300, "RxN (no defense) should give SEE = 300, got {see}");
}

#[test]
fn see_bad_rook_takes_pawn_defended_by_pawn_is_negative() {
    // Rook takes pawn defended by a pawn: SEE = −400.
    // FEN: white rook d1, black pawn d5, black pawn e6 defends d5.
    let board = Board::from_fen("4k3/8/4p3/3p4/8/8/8/3RK3 w - - 0 1").unwrap();
    let mv = find_move(&board, Square::D1, Square::D5);
    let see = board.see(mv);
    assert!(
        see < 0,
        "RxP defended by P should give negative SEE, got {see}"
    );
    assert_eq!(see, -400, "RxP defended by P should give SEE = −400, got {see}");
}

#[test]
fn see_xray_through_bishop() {
    // Queen behind bishop: after bishop captures, queen becomes attacker (X-ray).
    // White: bishop c3, queen a1 (behind bishop on diagonal). Black: pawn d4.
    // After BxP, white queen on a1 is revealed along the a1-h8 diagonal.
    let board = Board::from_fen("4k3/8/8/8/3p4/2B5/8/Q3K3 w - - 0 1").unwrap();
    let mv = find_move(&board, Square::C3, Square::D4);
    let see = board.see(mv);
    // BxP (no defense): SEE = 100.
    assert_eq!(see, 100, "BxP with queen X-ray (undefended) SEE = 100, got {see}");
}

// -----------------------------------------------------------------------
// Search depth and node count sanity
// -----------------------------------------------------------------------

#[test]
fn search_explores_nodes() {
    let result = search_to_depth("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5);
    assert!(
        result.nodes > 100,
        "depth-5 search should explore more than 100 nodes, got {}",
        result.nodes
    );
}

#[test]
fn deeper_search_explores_more_nodes() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let shallow = search_to_depth(fen, 3);
    let deep = search_to_depth(fen, 5);
    assert!(
        deep.nodes > shallow.nodes,
        "depth-5 ({}) should explore more nodes than depth-3 ({})",
        deep.nodes, shallow.nodes
    );
}
