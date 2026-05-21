use whitespine::board::moves::CAPTURE;
/// Correctness tests for the custom board representation.
///
/// Perft node counts are the gold standard for verifying move generation.
/// All expected values are taken from the Chess Programming Wiki.
use whitespine::board::{
    Bitboard, Board, Color, Move, Piece, Square, generate_legal_moves, mvv_lva, perft,
};

// -----------------------------------------------------------------------
// FEN round-trip
// -----------------------------------------------------------------------

#[test]
fn fen_round_trip_starting_position() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.to_fen(), fen);
}

#[test]
fn fen_round_trip_kiwipete() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.to_fen(), fen);
}

#[test]
fn fen_round_trip_endgame() {
    let fen = "8/2p5/3p4/KP5r/8/8/8/7k w - - 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.to_fen(), fen);
}

#[test]
fn fen_round_trip_with_ep() {
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.to_fen(), fen);
}

#[test]
fn fen_parse_invalid_rejects() {
    assert!(Board::from_fen("not a valid fen").is_err());
}

// -----------------------------------------------------------------------
// Basic board queries
// -----------------------------------------------------------------------

#[test]
fn starting_position_piece_counts() {
    let board = Board::starting_position();
    assert_eq!(board.pieces(Color::White, Piece::Pawn).count(), 8);
    assert_eq!(board.pieces(Color::White, Piece::Rook).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Knight).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Bishop).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Queen).count(), 1);
    assert_eq!(board.pieces(Color::White, Piece::King).count(), 1);
    assert_eq!(board.pieces(Color::Black, Piece::Pawn).count(), 8);
    assert_eq!(board.color_occ(Color::White).count(), 16);
    assert_eq!(board.color_occ(Color::Black).count(), 16);
    assert_eq!(board.all_occ.count(), 32);
}

#[test]
fn starting_position_king_squares() {
    let board = Board::starting_position();
    assert_eq!(board.king_sq(Color::White), Square::E1);
    assert_eq!(board.king_sq(Color::Black), Square::E8);
}

#[test]
fn starting_position_not_in_check() {
    let board = Board::starting_position();
    assert!(!board.is_in_check());
}

#[test]
fn check_detection() {
    // Bishop on d2 attacks white king on e1 diagonally
    let fen = "4k3/8/8/8/8/8/3b4/4K3 w - - 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert!(board.is_in_check());
}

#[test]
fn hash_consistent_after_make_unmake() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    let h0 = board.hash;
    let moves = generate_legal_moves(&board);
    for mv in moves {
        board.make_move(mv);
        board.unmake_move(mv);
        assert_eq!(board.hash, h0, "hash mismatch after make/unmake of {mv}");
    }
}

#[test]
fn hash_transposition_property() {
    // Two different paths to the same position should produce the same hash.
    // Use knight moves that are commutative — no EP, no castling changes.
    let mut b1 = Board::starting_position();
    let mut b2 = Board::starting_position();

    // Path 1: Nf3, d6, Nc3, e6
    b1.make_move(find_move(&b1, Square::G1, Square::F3));
    b1.make_move(find_move(&b1, Square::D7, Square::D6));
    b1.make_move(find_move(&b1, Square::B1, Square::C3));
    b1.make_move(find_move(&b1, Square::E7, Square::E6));

    // Path 2: Nc3, e6, Nf3, d6
    b2.make_move(find_move(&b2, Square::B1, Square::C3));
    b2.make_move(find_move(&b2, Square::E7, Square::E6));
    b2.make_move(find_move(&b2, Square::G1, Square::F3));
    b2.make_move(find_move(&b2, Square::D7, Square::D6));

    assert_eq!(b1.to_fen(), b2.to_fen());
    assert_eq!(
        b1.hash, b2.hash,
        "same position via different paths should have same hash"
    );
}

// -----------------------------------------------------------------------
// Move count tests
// -----------------------------------------------------------------------

#[test]
fn starting_position_has_20_moves() {
    let board = Board::starting_position();
    let moves = generate_legal_moves(&board);
    assert_eq!(
        moves.len(),
        20,
        "Starting position should have exactly 20 legal moves"
    );
}

#[test]
fn kiwipete_move_count() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_fen(fen).unwrap();
    let moves = generate_legal_moves(&board);
    assert_eq!(moves.len(), 48, "Kiwipete should have 48 legal moves");
}

#[test]
fn pos5_move_count() {
    // Position 5 from CPW (perft(1) = 44)
    let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let board = Board::from_fen(fen).unwrap();
    let moves = generate_legal_moves(&board);
    assert_eq!(moves.len(), 44, "CPW position 5 should have 44 legal moves");
}

#[test]
fn in_check_position_limited_moves() {
    // White king e1 in check from bishop d2 → only 5 escape moves
    let fen = "4k3/8/8/8/8/8/3b4/4K3 w - - 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert!(board.is_in_check());
    let moves = generate_legal_moves(&board);
    assert_eq!(
        moves.len(),
        5,
        "In-check position should have 5 legal moves"
    );
}

// -----------------------------------------------------------------------
// Perft tests — the gold standard for move generation correctness
// -----------------------------------------------------------------------

#[test]
fn perft_startpos_depth1() {
    let mut board = Board::starting_position();
    assert_eq!(perft(&mut board, 1), 20);
}

#[test]
fn perft_startpos_depth2() {
    let mut board = Board::starting_position();
    assert_eq!(perft(&mut board, 2), 400);
}

#[test]
fn perft_startpos_depth3() {
    let mut board = Board::starting_position();
    assert_eq!(perft(&mut board, 3), 8_902);
}

#[test]
fn perft_startpos_depth4() {
    let mut board = Board::starting_position();
    assert_eq!(perft(&mut board, 4), 197_281);
}

#[test]
fn perft_startpos_depth5() {
    let mut board = Board::starting_position();
    assert_eq!(perft(&mut board, 5), 4_865_609);
}

#[test]
fn perft_kiwipete_depth1() {
    let mut board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    assert_eq!(perft(&mut board, 1), 48);
}

#[test]
fn perft_kiwipete_depth2() {
    let mut board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    assert_eq!(perft(&mut board, 2), 2_039);
}

#[test]
fn perft_kiwipete_depth3() {
    let mut board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    assert_eq!(perft(&mut board, 3), 97_862);
}

#[test]
fn perft_pos3_depth5() {
    // Position 3: en passant heavy
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 5), 674_624);
}

#[test]
fn perft_pos4_depth4() {
    // Position 4 from CPW (white to move, depth 1 = 6, depth 4 = 422333)
    let mut board =
        Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
            .unwrap();
    assert_eq!(perft(&mut board, 4), 422_333);
}

#[test]
fn perft_pos5_depth4() {
    // Position 5 from CPW
    let mut board =
        Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    assert_eq!(perft(&mut board, 4), 2_103_487);
}

// -----------------------------------------------------------------------
// Move generation specific tests
// -----------------------------------------------------------------------

#[test]
fn en_passant_capture_works() {
    // White pawn on e5, black just played d5 → e.p. target is d6
    let mut board =
        Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2").unwrap();
    let moves = generate_legal_moves(&board);
    let ep_move = moves.iter().find(|&&mv| mv.is_en_passant()).copied();
    assert!(ep_move.is_some(), "en passant capture should be available");
    let mv = ep_move.unwrap();
    board.make_move(mv);
    // After EP: black pawn on d5 is gone, white pawn on d6
    assert_eq!(
        board.pieces(Color::White, Piece::Pawn) & Bitboard::from(Square::D6),
        Bitboard::from(Square::D6),
        "white pawn should be on d6 after EP"
    );
    assert!(
        board.pieces(Color::Black, Piece::Pawn) & Bitboard::from(Square::D5) == Bitboard::EMPTY,
        "black pawn on d5 should be gone after EP"
    );
}

#[test]
fn castling_kingside_white() {
    let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 4 4";
    let mut board = Board::from_fen(fen).unwrap();
    let moves = generate_legal_moves(&board);
    let castle = moves.iter().find(|&&mv| mv.is_castling()).copied();
    assert!(castle.is_some(), "kingside castling should be available");
    let mv = castle.unwrap();
    board.make_move(mv);
    // King should now be on G1, rook on F1
    assert_eq!(board.king_sq(Color::White), Square::G1);
    assert!(
        (board.pieces(Color::White, Piece::Rook) & Bitboard::from(Square::F1)).any(),
        "rook should be on F1 after kingside castle"
    );
}

#[test]
fn promotion_generates_four_moves() {
    // White pawn on d7, nothing on d8 or e8, can promote to d8
    let fen = "5k2/3P4/8/8/8/8/8/4K3 w - - 0 1";
    let board = Board::from_fen(fen).unwrap();
    let moves = generate_legal_moves(&board);
    let promos: Vec<_> = moves.iter().filter(|&&mv| mv.is_promo()).collect();
    assert_eq!(
        promos.len(),
        4,
        "should generate 4 promotion moves (Q, R, B, N)"
    );
}

// -----------------------------------------------------------------------
// Helper
// -----------------------------------------------------------------------

fn find_move(board: &Board, from: Square, to: Square) -> Move {
    generate_legal_moves(board)
        .into_iter()
        .find(|mv| mv.from_sq() == from && mv.to_sq() == to)
        .unwrap_or_else(|| panic!("no move from {from} to {to}"))
}

// -----------------------------------------------------------------------
// Null move
// -----------------------------------------------------------------------

#[test]
fn null_move_flips_side_to_move() {
    let mut board = Board::starting_position();
    assert_eq!(board.side_to_move, Color::White);
    board.make_null_move();
    assert_eq!(board.side_to_move, Color::Black);
    board.unmake_null_move();
    assert_eq!(board.side_to_move, Color::White);
}

#[test]
fn null_move_hash_restores() {
    let mut board = Board::starting_position();
    let orig_hash = board.hash;
    board.make_null_move();
    // hash changed (side bit flipped)
    assert_ne!(board.hash, orig_hash);
    board.unmake_null_move();
    // hash fully restored
    assert_eq!(board.hash, orig_hash);
}

#[test]
fn null_move_clears_ep_square() {
    // After 1.e4 it's Black's turn; EP square = e3 (index 20)
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    let ep_before = board.ep_square().map(|s| s.index()).unwrap_or(255);
    board.make_null_move();
    // EP square must be cleared after passing the turn
    assert!(
        board.ep_square().is_none(),
        "EP square should be cleared after null move"
    );
    board.unmake_null_move();
    // EP restored
    assert_eq!(
        board.ep_square().map(|s| s.index()).unwrap_or(255),
        ep_before,
        "EP square should be restored after unmake_null_move"
    );
}

// -----------------------------------------------------------------------
// Repetition detection
// -----------------------------------------------------------------------

#[test]
fn is_repetition_detects_draw() {
    // Knight moves: Ng1-f3-g1 twice → same position appears again.
    let mut board = Board::starting_position();
    // Move 1 — Ng1-f3
    let mv1 = find_move(&board, Square::G1, Square::F3);
    board.make_move(mv1);
    // Move 1 — Ng8-f6
    let mv2 = find_move(&board, Square::G8, Square::F6);
    board.make_move(mv2);
    // Move 2 — Nf3-g1
    let mv3 = find_move(&board, Square::F3, Square::G1);
    board.make_move(mv3);
    // Move 2 — Nf6-g8
    let mv4 = find_move(&board, Square::F6, Square::G8);
    board.make_move(mv4);
    // We are back to the starting position — should be a repetition!
    assert!(
        board.is_repetition(),
        "position should be detected as a repetition after both knights returned home"
    );
}

#[test]
fn no_repetition_after_irreversible_move() {
    let mut board = Board::starting_position();
    // e2-e4 is irreversible (pawn move)
    let mv = find_move(&board, Square::E2, Square::E4);
    board.make_move(mv);
    // No previous position can be a repetition now
    assert!(!board.is_repetition());
}

// -----------------------------------------------------------------------
// Material balance
// -----------------------------------------------------------------------

#[test]
fn material_tracking_startpos() {
    // Starting position is perfectly balanced — White and Black have equal material.
    let board = Board::starting_position();
    assert_eq!(
        board.material, 0,
        "starting position should have zero material imbalance"
    );
}

#[test]
fn material_tracking_after_capture() {
    let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
    let mut board = Board::from_fen(fen).unwrap();
    let initial_material = board.material;
    // White captures on d5 with e4xd5
    let cap = find_move(&mut board, Square::E4, Square::D5);
    board.make_move(cap);
    // White up one pawn (+100)
    assert_eq!(
        board.material,
        initial_material + 100,
        "material should increase by 100 after White captures a Black pawn"
    );
    board.unmake_move(cap);
    assert_eq!(
        board.material, initial_material,
        "material should be restored after unmake"
    );
}

// -----------------------------------------------------------------------
// MVV-LVA ordering
// -----------------------------------------------------------------------

#[test]
fn mvv_lva_queen_takes_pawn_less_than_pawn_takes_queen() {
    // White: Qd1, Pe4, Ke1 | Black: Qd5, Pe7, Ke8
    let fen = "4k3/4p3/8/3q4/4P3/8/8/3QK3 w - - 0 1";
    let board = Board::from_fen(fen).unwrap();

    // e4xd5 (pawn captures queen)
    let pawn_cap = Move::new(Square::E4, Square::D5, CAPTURE);
    // Qd1xd5 (queen captures queen)
    let queen_cap = Move::new(Square::D1, Square::D5, CAPTURE);

    let pawn_score = mvv_lva(&board, pawn_cap);
    let queen_score = mvv_lva(&board, queen_cap);

    assert!(
        pawn_score > queen_score,
        "pawn capturing queen ({pawn_score}) should rank higher than queen capturing queen ({queen_score})"
    );
}
