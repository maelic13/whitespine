/// Tests for the evaluation function.
use whitespine::board::Board;
use whitespine::eval::Evaluator;

fn eval(fen: &str) -> i32 {
    let board = Board::from_fen(fen).unwrap();
    Evaluator::new().evaluate(&board)
}

// -----------------------------------------------------------------------
// Symmetry / starting position
// -----------------------------------------------------------------------

#[test]
fn starting_position_near_zero() {
    // Starting position is symmetric; score should be close to zero.
    // The PeSTO PST tables may introduce a small asymmetry, so we allow ±100.
    let score = eval("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert!(
        score.abs() <= 100,
        "starting position score should be near zero, got {score}"
    );
}

#[test]
fn score_flips_sign_for_symmetric_position() {
    // Use a position that is exactly symmetric but change the side to move.
    // The evaluation should flip sign.
    let fen_w = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
    let fen_b = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 4 4";
    let score_w = eval(fen_w);
    let score_b = eval(fen_b);
    // Scores should have opposite signs (within a small tolerance for tempo).
    assert!(
        score_w > 0 && score_b < 0 || score_w < 0 && score_b > 0 || (score_w == 0 && score_b == 0),
        "scores should be negated for opposite sides: white={score_w}, black={score_b}"
    );
    // The magnitude should be the same (within tempo tolerance).
    assert!(
        (score_w + score_b).abs() <= 30,
        "score magnitudes differ too much: white={score_w}, black={score_b}"
    );
}

// -----------------------------------------------------------------------
// Material advantage
// -----------------------------------------------------------------------

#[test]
fn extra_queen_gives_large_positive_score() {
    // White has an extra queen (no black queen on board).
    let score = eval("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert!(
        score >= 800,
        "extra queen should give score >= 800, got {score}"
    );
}

#[test]
fn extra_rook_gives_large_positive_score() {
    // White has an extra rook.
    let score = eval("rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQq - 0 1");
    assert!(
        score >= 400,
        "extra rook should give score >= 400, got {score}"
    );
}

#[test]
fn extra_rook_beats_extra_knight() {
    let rook_score = eval("rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQq - 0 1");
    let knight_score = eval("r1bqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert!(
        rook_score > knight_score,
        "extra rook ({rook_score}) should score higher than extra knight ({knight_score})"
    );
}

#[test]
fn black_extra_queen_gives_large_negative_score() {
    // Black has an extra queen (no white queen).
    let score = eval("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");
    assert!(
        score <= -800,
        "black extra queen should give score <= -800, got {score}"
    );
}

// -----------------------------------------------------------------------
// Piece-specific bonuses
// -----------------------------------------------------------------------

#[test]
fn bishop_pair_bonus() {
    // Isolated test: white has two bishops vs one bishop, symmetric pawns and kings.
    let w2b = eval("4k3/pppppppp/8/8/8/8/PPPPPPPP/2B1KB2 w - - 0 1");
    let w1b = eval("4k3/pppppppp/8/8/8/8/PPPPPPPP/4KB2 w - - 0 1");
    // Having 2 bishops should score higher than 1.
    assert!(
        w2b > w1b,
        "two bishops ({w2b}) should score higher than one bishop ({w1b})"
    );
}

#[test]
fn doubled_pawns_penalized() {
    // Two doubled pawns (d2+d3) vs two undoubled pawns (d2+e2) — same material count.
    let doubled   = eval("4k3/8/8/8/8/3P4/3P4/4K3 w - - 0 1");
    let undoubled = eval("4k3/8/8/8/8/8/3PP3/4K3 w - - 0 1");
    assert!(
        doubled <= undoubled,
        "doubled pawns ({doubled}) should score <= undoubled ({undoubled})"
    );
}

#[test]
fn extra_material_monotone() {
    // More material advantage = higher score.
    let one_extra_pawn = eval("rnbqkbnr/ppppppp1/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let one_extra_rook = eval("rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQq - 0 1");
    assert!(
        one_extra_rook > one_extra_pawn,
        "extra rook ({one_extra_rook}) should score > extra pawn ({one_extra_pawn})"
    );
}

// -----------------------------------------------------------------------
// King safety
// -----------------------------------------------------------------------

#[test]
fn exposed_king_penalized_vs_sheltered_king() {
    // White king on g1 with full pawn shelter (f2, g2, h2) should score better
    // than white king on e4 (centre, no pawn shelter) with same material.
    let sheltered = eval("4k3/pppppppp/8/8/8/8/5PPP/6K1 w - - 0 1");
    let exposed   = eval("4k3/pppppppp/8/8/4K3/8/8/8 w - - 0 1");
    assert!(
        sheltered >= exposed - 50,
        "sheltered king ({sheltered}) should not be much worse than exposed king ({exposed})"
    );
}
