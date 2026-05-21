use std::collections::{BTreeMap, BTreeSet};

use whitespine::board::{Bitboard, Board, Color, GameResult, Piece, STARTING_FEN, Square};

const ORACLE_FENS: &[&str] = &[
    STARTING_FEN,
    // CPW position 2 / Kiwipete: castling, pins, captures and en-passant.
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    // CPW position 3: en-passant and endgame checks.
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    // CPW position 4 and its mirror: promotions, castling and checks.
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
    // CPW positions 5 and 6.
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    // Search benchmark and edge-case positions.
    "rnbq1k1r/pppp1ppp/4pn2/8/1b1PP3/2N2N2/PPP2PPP/R1BQKB1R w KQ - 2 5",
    "rnbqkb1r/pppp1ppp/5n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 3 3",
    "8/8/3p4/KPp4r/8/8/8/7k w - c6 0 1",
    "4k3/P6P/8/8/8/8/p6p/4K3 w - - 0 1",
    "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    "r3k2r/8/8/8/8/5r2/8/R3K2R w KQkq - 0 1",
    "4k3/8/8/8/8/2b5/4r3/4K3 w - - 0 1",
];

#[test]
fn starting_position_legal_moves_are_exact() {
    let board = Board::starting_position();
    assert_eq!(
        custom_move_set(&board),
        move_set(&[
            "a2a3", "a2a4", "b1a3", "b1c3", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4", "e2e3",
            "e2e4", "f2f3", "f2f4", "g1f3", "g1h3", "g2g3", "g2g4", "h2h3", "h2h4",
        ])
    );
}

#[test]
fn special_position_move_sets_are_exact_for_key_rules() {
    let castles = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    let moves = custom_move_set(&castles);
    assert!(moves.contains("e1g1"));
    assert!(moves.contains("e1c1"));
    assert!(moves.contains("a1a8"));
    assert!(moves.contains("h1h8"));

    let promotions = Board::from_fen("4k3/P6P/8/8/8/8/p6p/4K3 w - - 0 1").unwrap();
    let moves = custom_move_set(&promotions);
    for mv in [
        "a7a8q", "a7a8r", "a7a8b", "a7a8n", "h7h8q", "h7h8r", "h7h8b", "h7h8n",
    ] {
        assert!(moves.contains(mv), "{mv} missing from promotion set");
    }

    let ep = Board::from_fen("8/8/8/3pP3/8/8/8/4K2k w - d6 0 1").unwrap();
    assert!(custom_move_set(&ep).contains("e5d6"));
}

#[test]
fn legal_captures_are_exact_for_curated_positions() {
    let cases = [
        ("8/8/8/3pP3/8/8/8/4K2k w - d6 0 1", &["e5d6"][..]),
        ("4k3/8/8/8/8/2b5/4r3/4K3 w - - 0 1", &["e1e2"][..]),
        ("4k3/8/8/8/3q4/2N1B3/8/4K3 w - - 0 1", &["e3d4"][..]),
    ];

    for (fen, expected) in cases {
        let mut board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        let moves = board
            .generate_legal_captures()
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert_eq!(moves, move_set(expected), "capture set differs for {fen}");
    }
}

#[test]
fn legal_quiets_and_captures_partition_all_legal_moves() {
    for fen in ORACLE_FENS {
        let board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        let all = board
            .generate_legal_moves()
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let quiets = board
            .generate_legal_quiets()
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let mut capture_board = board.clone();
        let captures = capture_board
            .generate_legal_captures()
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();

        assert!(
            quiets.is_disjoint(&captures),
            "quiet/capture sets overlap for {fen}"
        );
        assert_eq!(
            quiets.union(&captures).cloned().collect::<BTreeSet<_>>(),
            all,
            "quiet/capture partition differs for {fen}"
        );
    }
}

#[test]
fn check_detection_matches_hand_checked_positions() {
    let cases = [
        ("4k3/8/8/8/8/2b5/4r3/4K3 w - - 0 1", true, 2),
        ("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1", true, 1),
        ("4k3/8/8/8/8/8/8/4K3 w - - 0 1", false, 0),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            false,
            0,
        ),
    ];

    for (fen, expected_check, expected_checkers) in cases {
        let board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        assert_eq!(board.is_in_check(), expected_check, "check state for {fen}");
        assert_eq!(
            board.checkers().count(),
            expected_checkers,
            "checkers for {fen}"
        );
    }
}

#[test]
fn fen_round_trip_preserves_state() {
    for fen in ORACLE_FENS {
        let board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        let reparsed = Board::from_fen(&board.to_fen()).unwrap();

        assert_eq!(board.to_fen(), reparsed.to_fen(), "FEN round-trip failed");
        assert_eq!(
            board.hash, reparsed.hash,
            "hash round-trip failed for {fen}"
        );
        assert_eq!(
            board.is_in_check(),
            reparsed.is_in_check(),
            "check state round-trip failed for {fen}"
        );
    }
}

#[test]
fn perft_matches_common_reference_counts() {
    let cases = [
        (STARTING_FEN, 0, 1),
        (STARTING_FEN, 1, 20),
        (STARTING_FEN, 2, 400),
        (STARTING_FEN, 3, 8_902),
        (STARTING_FEN, 4, 197_281),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            1,
            48,
        ),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            2,
            2_039,
        ),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            3,
            97_862,
        ),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 3, 2_812),
        (
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            3,
            9_467,
        ),
        (
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            3,
            62_379,
        ),
        (
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            3,
            89_890,
        ),
    ];

    for (fen, depth, expected) in cases {
        let mut board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        assert_eq!(
            board.perft(depth),
            expected,
            "perft({depth}) failed for {fen}"
        );
    }
}

#[test]
fn deeper_perft_reference_counts_in_release() {
    if cfg!(debug_assertions) {
        return;
    }

    let cases = [
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            4,
            4_085_603,
        ),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 4, 43_238),
        (
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            4,
            422_333,
        ),
        (
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            4,
            2_103_487,
        ),
        (
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            4,
            3_894_594,
        ),
    ];

    for (fen, depth, expected) in cases {
        let mut board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        assert_eq!(
            board.perft(depth),
            expected,
            "release perft({depth}) failed for {fen}"
        );
    }
}

#[test]
fn perft_divide_matches_known_start_position_depth_two() {
    let mut board = Board::starting_position();
    let expected = [
        ("a2a3", 20),
        ("a2a4", 20),
        ("b1a3", 20),
        ("b1c3", 20),
        ("b2b3", 20),
        ("b2b4", 20),
        ("c2c3", 20),
        ("c2c4", 20),
        ("d2d3", 20),
        ("d2d4", 20),
        ("e2e3", 20),
        ("e2e4", 20),
        ("f2f3", 20),
        ("f2f4", 20),
        ("g1f3", 20),
        ("g1h3", 20),
        ("g2g3", 20),
        ("g2g4", 20),
        ("h2h3", 20),
        ("h2h4", 20),
    ]
    .into_iter()
    .map(|(mv, count)| (mv.to_string(), count))
    .collect::<BTreeMap<_, _>>();

    assert_eq!(custom_perft_divide(&mut board, 2), expected);
}

#[test]
fn make_unmake_restores_state_through_two_ply_walk() {
    for fen in ORACLE_FENS {
        let mut board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        assert_make_unmake_stable(&mut board, 2);
    }
}

#[test]
fn move_generation_does_not_mutate_board_state() {
    for fen in ORACLE_FENS {
        let mut board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        let original = Snapshot::from(&board);

        let _ = board.generate_legal_moves();
        original.assert_same(&board, "legal move generation");

        let _ = board.generate_legal_captures();
        original.assert_same(&board, "capture generation");

        let _ = board.is_in_check();
        let _ = board.checkers();
        original.assert_same(&board, "check-state query");
    }
}

#[test]
fn special_moves_update_board_and_hash_correctly() {
    let mut castle = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    let before_castle = Snapshot::from(&castle);
    let white_king_side = castle.parse_move("e1g1").unwrap();
    castle.make_move_unchecked(white_king_side);
    assert_eq!(castle.to_fen(), "r3k2r/8/8/8/8/8/8/R4RK1 b kq - 1 1");
    castle.unmake_move(white_king_side);
    before_castle.assert_same(&castle, "castling unmake");

    let mut ep = Board::from_fen("8/8/8/3pP3/8/8/8/4K2k w - d6 0 1").unwrap();
    let before_ep = Snapshot::from(&ep);
    let ep_capture = ep.parse_move("e5d6").unwrap();
    ep.make_move_unchecked(ep_capture);
    assert_eq!(ep.to_fen(), "8/8/3P4/8/8/8/8/4K2k b - - 0 1");
    ep.unmake_move(ep_capture);
    before_ep.assert_same(&ep, "en-passant unmake");

    let mut promotion = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let before_promotion = Snapshot::from(&promotion);
    let promote = promotion.parse_move("a7a8q").unwrap();
    promotion.make_move_unchecked(promote);
    assert_eq!(
        promotion.piece_at(Square::A8),
        Some((Color::White, Piece::Queen))
    );
    assert_eq!(promotion.to_fen(), "Q3k3/8/8/8/8/8/8/4K3 b - - 0 1");
    promotion.unmake_move(promote);
    before_promotion.assert_same(&promotion, "promotion unmake");
}

#[test]
fn illegal_edge_case_moves_are_excluded() {
    let ep_pin = Board::from_fen("8/8/3p4/KPp4r/8/8/8/7k w - c6 0 1").unwrap();
    assert!(
        !custom_move_set(&ep_pin).contains("b5c6"),
        "en-passant exposing a horizontal rook check must be illegal"
    );

    let castle_through_check = Board::from_fen("r3k2r/8/8/8/8/5r2/8/R3K2R w KQkq - 0 1").unwrap();
    assert!(
        !custom_move_set(&castle_through_check).contains("e1g1"),
        "castling through an attacked transit square must be illegal"
    );
}

#[derive(Debug)]
struct Snapshot {
    fen: String,
    hash: u64,
    in_check: bool,
    checkers: Bitboard,
}

impl Snapshot {
    fn from(board: &Board) -> Self {
        Self {
            fen: board.to_fen(),
            hash: board.hash,
            in_check: board.is_in_check(),
            checkers: board.checkers(),
        }
    }

    fn assert_same(&self, board: &Board, context: &str) {
        assert_eq!(board.to_fen(), self.fen, "{context} changed FEN");
        assert_eq!(board.hash, self.hash, "{context} changed hash");
        assert_eq!(
            board.is_in_check(),
            self.in_check,
            "{context} changed check state"
        );
        assert_eq!(
            board.checkers(),
            self.checkers,
            "{context} changed checker bitboard"
        );
    }
}

fn assert_make_unmake_stable(board: &mut Board, depth: u32) {
    let before = Snapshot::from(board);
    if depth == 0 {
        return;
    }

    let moves = board.generate_legal_moves();
    for mv in moves {
        board.make_move_unchecked(mv);
        assert_make_unmake_stable(board, depth - 1);
        board.unmake_move(mv);
        before.assert_same(board, &format!("make/unmake of {mv}"));
    }
}

fn custom_move_set(board: &Board) -> BTreeSet<String> {
    board
        .generate_legal_moves()
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn move_set(moves: &[&str]) -> BTreeSet<String> {
    moves.iter().map(|mv| mv.to_string()).collect()
}

fn custom_perft_divide(board: &mut Board, depth: u32) -> BTreeMap<String, u64> {
    let mut divide = BTreeMap::new();
    for mv in board.generate_legal_moves() {
        board.make_move_unchecked(mv);
        let nodes = board.perft(depth - 1);
        board.unmake_move(mv);
        divide.insert(mv.to_string(), nodes);
    }
    divide
}

// -----------------------------------------------------------------------
// Piece accessor tests
// -----------------------------------------------------------------------

#[test]
fn piece_accessors_at_starting_position() {
    let board = Board::starting_position();

    // White pieces
    assert_eq!(board.pieces(Color::White, Piece::Pawn).count(), 8);
    assert_eq!(board.pieces(Color::White, Piece::Knight).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Bishop).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Rook).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Queen).count(), 1);
    assert_eq!(board.pieces(Color::White, Piece::King).count(), 1);

    // Black pieces
    assert_eq!(board.pieces(Color::Black, Piece::Pawn).count(), 8);
    assert_eq!(board.pieces(Color::Black, Piece::Knight).count(), 2);
    assert_eq!(board.pieces(Color::Black, Piece::Bishop).count(), 2);
    assert_eq!(board.pieces(Color::Black, Piece::Rook).count(), 2);
    assert_eq!(board.pieces(Color::Black, Piece::Queen).count(), 1);
    assert_eq!(board.pieces(Color::Black, Piece::King).count(), 1);

    // Total occupancy
    assert_eq!(board.occupied_count(), 32);
    assert_eq!(board.color_occ(Color::White).count(), 16);
    assert_eq!(board.color_occ(Color::Black).count(), 16);

    // Spot checks via mailbox
    assert_eq!(
        board.piece_at(Square::E1),
        Some((Color::White, Piece::King))
    );
    assert_eq!(
        board.piece_at(Square::D1),
        Some((Color::White, Piece::Queen))
    );
    assert_eq!(
        board.piece_at(Square::E8),
        Some((Color::Black, Piece::King))
    );
    assert_eq!(board.piece_at(Square::E4), None);

    // King squares
    assert_eq!(board.king_sq(Color::White), Square::E1);
    assert_eq!(board.king_sq(Color::Black), Square::E8);
}

// -----------------------------------------------------------------------
// Game result tests
// -----------------------------------------------------------------------

#[test]
fn game_result_ongoing_returns_none() {
    let board = Board::starting_position();
    assert_eq!(
        board.game_result(),
        None,
        "starting position must be ongoing"
    );

    // A mid-game position should also be ongoing
    let board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    assert_eq!(board.game_result(), None, "kiwipete must be ongoing");
}

#[test]
fn game_result_checkmate_fool_s_mate() {
    // Fool's mate: 1.f3 e5 2.g4 Qh4# — white is checkmated
    let mut board = Board::starting_position();
    for mv in ["f2f3", "e7e5", "g2g4", "d8h4"] {
        assert!(board.play_uci(mv), "move {mv} must be legal");
    }
    assert!(board.is_in_check(), "white must be in check after Qh4");
    assert_eq!(
        board.game_result(),
        Some(GameResult::BlackCheckmates),
        "Fool's mate must be BlackCheckmates"
    );
}

#[test]
fn game_result_stalemate_detected() {
    // Black king on e8, white pawn on e7, white king on e6: black to move, stalemate
    let board = Board::from_fen("4k3/4P3/4K3/8/8/8/8/8 b - - 0 1").unwrap();
    assert!(
        !board.is_in_check(),
        "stalemate position must not be in check"
    );
    assert_eq!(
        board.generate_legal_moves().len(),
        0,
        "black must have no legal moves"
    );
    assert_eq!(
        board.game_result(),
        Some(GameResult::Stalemate),
        "must be stalemate"
    );
}

// -----------------------------------------------------------------------
// Insufficient material / draw detection tests
// -----------------------------------------------------------------------

#[test]
fn insufficient_material_draw_cases() {
    // KK — only kings
    let kk = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    assert!(kk.can_declare_draw(), "KK must be insufficient material");

    // KNK — one knight
    let knk = Board::from_fen("4k3/8/8/8/8/8/8/4KN2 w - - 0 1").unwrap();
    assert!(knk.can_declare_draw(), "KNK must be insufficient material");

    // KBK — one bishop
    let kbk = Board::from_fen("4k3/8/8/8/8/8/3B4/4K3 w - - 0 1").unwrap();
    assert!(kbk.can_declare_draw(), "KBK must be insufficient material");

    // KBBvK — two bishops on same color complex (both dark: e2 complex=1, f1 complex=1)
    let kbbk_same = Board::from_fen("4k3/8/8/8/8/8/4B3/4KB2 w - - 0 1").unwrap();
    assert!(
        kbbk_same.can_declare_draw(),
        "KBBvK with same-color bishops must be insufficient material"
    );
}

#[test]
fn insufficient_material_not_draw_cases() {
    // KPK — pawn present
    let kpk = Board::from_fen("4k3/p7/8/8/8/8/8/4K3 b - - 0 1").unwrap();
    assert!(
        !kpk.can_declare_draw(),
        "KPK must not be insufficient material"
    );

    // KQK — queen present
    let kqk = Board::from_fen("4k3/8/8/8/8/8/8/4KQ2 w - - 0 1").unwrap();
    assert!(
        !kqk.can_declare_draw(),
        "KQK must not be insufficient material"
    );

    // KRK — rook present
    let krk = Board::from_fen("4k3/8/8/8/8/8/8/R3K3 w - - 0 1").unwrap();
    assert!(
        !krk.can_declare_draw(),
        "KRK must not be insufficient material"
    );

    // KBBvK — two bishops on different color complexes: c1(complex=0), e2(complex=1)
    let kbbk_diff = Board::from_fen("4k3/8/8/8/8/8/4B3/2B1K3 w - - 0 1").unwrap();
    assert!(
        !kbbk_diff.can_declare_draw(),
        "KBBvK with opposite-color bishops must not be draw by material alone"
    );

    // KNNvK — two knights (mating possibility exists theoretically)
    let knnk = Board::from_fen("4k3/8/8/8/8/8/8/3NKN2 w - - 0 1").unwrap();
    assert!(
        !knnk.can_declare_draw(),
        "KNNvK must not be insufficient material"
    );
}

// -----------------------------------------------------------------------
// Threefold repetition / 50-move draw
// -----------------------------------------------------------------------

#[test]
fn threefold_repetition_triggers_can_declare_draw() {
    let mut board = Board::starting_position();
    assert!(!board.can_declare_draw(), "initial position is not a draw");

    // Shuffle knights back and forth: after 8 half-moves the starting hash
    // has appeared in history twice, giving a total count of 3 (threefold).
    let cycle = ["g1f3", "g8f6", "f3g1", "f6g8"];
    for mv in cycle.iter().chain(cycle.iter()) {
        assert!(board.play_uci(mv), "move {mv} must be legal");
    }

    assert!(
        board.can_declare_draw(),
        "position must be threefold repetition after two full knight cycles"
    );
}

#[test]
fn fifty_move_rule_triggers_can_declare_draw() {
    // Start from a position where we can make 100 quiet (non-pawn, non-capture) moves.
    // Rook and king shuttle without pawn moves or captures until the clock reaches 100.
    let mut board = Board::from_fen("4k3/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();
    // 25 four-ply cycles = 100 half-moves.
    for _ in 0..25 {
        assert!(board.play_uci("a1a2"), "Ra2 must be legal");
        assert!(board.play_uci("e8e7"), "Ke7 must be legal");
        assert!(board.play_uci("a2a1"), "Ra1 must be legal");
        assert!(board.play_uci("e7e8"), "Ke8 must be legal");
    }
    assert_eq!(board.halfmove_clock, 100, "halfmove clock must reach 100");
    assert!(board.can_declare_draw(), "50-move rule must trigger");
}

// -----------------------------------------------------------------------
// Clock tracking
// -----------------------------------------------------------------------

#[test]
fn halfmove_clock_tracking() {
    let mut board = Board::starting_position();
    assert_eq!(board.halfmove_clock, 0);

    // Knight move: increments clock
    board.play_uci("g1f3");
    assert_eq!(
        board.halfmove_clock, 1,
        "quiet knight move must increment clock"
    );

    board.play_uci("g8f6");
    assert_eq!(
        board.halfmove_clock, 2,
        "quiet knight move must increment clock"
    );

    // Pawn move: resets clock
    board.play_uci("e2e4");
    assert_eq!(
        board.halfmove_clock, 0,
        "pawn move must reset halfmove clock"
    );

    board.play_uci("d7d5");
    assert_eq!(
        board.halfmove_clock, 0,
        "pawn move must reset halfmove clock"
    );

    // Capture: resets clock (e4 captures d5)
    board.play_uci("e4d5");
    assert_eq!(board.halfmove_clock, 0, "capture must reset halfmove clock");
}

#[test]
fn fullmove_counter_tracking() {
    let mut board = Board::starting_position();
    assert_eq!(board.fullmove, 1);

    board.play_uci("e2e4"); // white's move — still move 1
    assert_eq!(
        board.fullmove, 1,
        "fullmove must not change after white's move"
    );

    board.play_uci("e7e5"); // black's move — now move 2
    assert_eq!(
        board.fullmove, 2,
        "fullmove must increment after black's move"
    );

    board.play_uci("g1f3");
    assert_eq!(board.fullmove, 2);

    board.play_uci("g8f6");
    assert_eq!(
        board.fullmove, 3,
        "fullmove must increment after each black move"
    );
}

// -----------------------------------------------------------------------
// Castling rights
// -----------------------------------------------------------------------

#[test]
fn castling_rights_cleared_by_king_move() {
    let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    use whitespine::board::CastlingRights;

    assert!(board.castling.has(CastlingRights::WHITE_KINGSIDE));
    assert!(board.castling.has(CastlingRights::WHITE_QUEENSIDE));

    // White king moves to d1 — both white castling rights must be cleared
    board.play_uci("e1d1");
    assert!(
        !board.castling.has(CastlingRights::WHITE_KINGSIDE),
        "WK right must be cleared after king move"
    );
    assert!(
        !board.castling.has(CastlingRights::WHITE_QUEENSIDE),
        "WQ right must be cleared after king move"
    );
    // Black rights must be untouched
    assert!(board.castling.has(CastlingRights::BLACK_KINGSIDE));
    assert!(board.castling.has(CastlingRights::BLACK_QUEENSIDE));
}

#[test]
fn castling_rights_cleared_by_rook_move() {
    let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    use whitespine::board::CastlingRights;

    // White h1 rook moves — only WHITE_KINGSIDE should be cleared
    board.play_uci("h1h2");
    assert!(
        !board.castling.has(CastlingRights::WHITE_KINGSIDE),
        "WK right must be cleared when h1 rook moves"
    );
    assert!(
        board.castling.has(CastlingRights::WHITE_QUEENSIDE),
        "WQ right must be intact"
    );
    assert!(board.castling.has(CastlingRights::BLACK_KINGSIDE));
    assert!(board.castling.has(CastlingRights::BLACK_QUEENSIDE));
}

#[test]
fn castling_rights_cleared_when_rook_is_captured() {
    // Black rook captures white a1 rook — should clear WHITE_QUEENSIDE
    let mut board = Board::from_fen("r3k3/8/8/8/8/8/8/R3K3 b Q - 0 1").unwrap();
    use whitespine::board::CastlingRights;

    board.play_uci("a8a1");
    assert!(
        !board.castling.has(CastlingRights::WHITE_QUEENSIDE),
        "WQ right must be cleared when a1 rook is captured"
    );
}

// -----------------------------------------------------------------------
// Zobrist hash transposition property
// -----------------------------------------------------------------------

#[test]
fn zobrist_transposition_same_position_same_hash() {
    // Two different move orders arriving at the same position must yield the
    // same Zobrist hash and the same FEN.
    //
    // Path A: 1.Nf3 Nf6 2.Nc3 Nc6
    // Path B: 1.Nc3 Nc6 2.Nf3 Nf6  (same position via different move order)
    let mut path_a = Board::starting_position();
    for mv in ["g1f3", "g8f6", "b1c3", "b8c6"] {
        assert!(path_a.play_uci(mv), "Path A move {mv} must be legal");
    }

    let mut path_b = Board::starting_position();
    for mv in ["b1c3", "b8c6", "g1f3", "g8f6"] {
        assert!(path_b.play_uci(mv), "Path B move {mv} must be legal");
    }

    assert_eq!(
        path_a.to_fen(),
        path_b.to_fen(),
        "both paths must yield the same FEN"
    );
    assert_eq!(
        path_a.hash, path_b.hash,
        "both paths must yield the same Zobrist hash"
    );
}

#[test]
fn zobrist_different_positions_have_different_hashes() {
    let start = Board::starting_position();
    let mut after_e4 = Board::starting_position();
    after_e4.play_uci("e2e4");
    let mut after_d4 = Board::starting_position();
    after_d4.play_uci("d2d4");

    assert_ne!(start.hash, after_e4.hash, "start vs 1.e4 must differ");
    assert_ne!(start.hash, after_d4.hash, "start vs 1.d4 must differ");
    assert_ne!(after_e4.hash, after_d4.hash, "1.e4 vs 1.d4 must differ");
}

// -----------------------------------------------------------------------
// En passant square tracking
// -----------------------------------------------------------------------

#[test]
fn ep_square_set_after_double_push_and_cleared_afterwards() {
    let mut board = Board::starting_position();
    assert_eq!(board.ep_square(), None, "no EP at start");

    // Double pawn push sets EP target square (the square the capturing pawn lands on)
    board.play_uci("e2e4");
    assert_eq!(
        board.ep_square(),
        Some(Square::E3),
        "EP square must be e3 after 1.e4"
    );

    // Any non-EP move clears the EP square
    board.play_uci("e7e5");
    assert_eq!(
        board.ep_square(),
        Some(Square::E6),
        "EP square must be e6 after 1...e5"
    );

    board.play_uci("g1f3"); // quiet move — no double push
    assert_eq!(
        board.ep_square(),
        None,
        "EP square must be cleared after non-double-push move"
    );
}

#[test]
fn ep_capture_removes_pawn_from_correct_square() {
    // After 1.e4 d5 2.e5 f5, the en passant capture e5xf6 removes the f5 pawn
    let mut board = Board::starting_position();
    for mv in ["e2e4", "d7d5", "e4e5", "f7f5"] {
        assert!(board.play_uci(mv), "move {mv} must be legal");
    }
    assert_eq!(
        board.ep_square(),
        Some(Square::F6),
        "EP target must be f6 after 2...f5"
    );

    // Execute the en passant capture
    let ep_mv = board.parse_move("e5f6").expect("e5f6 must be legal");
    assert!(ep_mv.is_en_passant(), "e5f6 must be flagged as EP");
    board.make_move_unchecked(ep_mv);

    // The captured pawn (on f5) must be gone; the capturing pawn must be on f6
    assert_eq!(
        board.piece_at(Square::F6),
        Some((Color::White, Piece::Pawn)),
        "white pawn must land on f6"
    );
    assert_eq!(
        board.piece_at(Square::F5),
        None,
        "captured pawn on f5 must be removed"
    );
    assert_eq!(
        board.ep_square(),
        None,
        "EP square must be cleared after EP capture"
    );
}
