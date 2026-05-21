/// Legal move generation and perft.
///
/// Strategy:
///   1. Find the king square, compute checkers and pinned pieces.
///   2. In double check: generate only king evasions.
///   3. In single check: generate king moves + interpositions/captures of checker.
///   4. No check: generate all moves for each piece, respecting pins.
use std::sync::LazyLock;

use super::attacks::ATTACKS;
use super::bitboard::Bitboard;
use super::board::Board;
use super::moves::{
    CAPTURE, CASTLE_KINGSIDE, CASTLE_QUEENSIDE, DOUBLE_PUSH, EN_PASSANT, Move, MoveList,
    PROMO_BISHOP, PROMO_CAPTURE_BISHOP, PROMO_CAPTURE_KNIGHT, PROMO_CAPTURE_QUEEN,
    PROMO_CAPTURE_ROOK, PROMO_KNIGHT, PROMO_QUEEN, PROMO_ROOK, QUIET,
};
use super::piece::{CastlingRights, Color, Piece};
use super::square::{Rank, Square};

// -----------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------

/// Generate all legal moves for the current position.
pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(48);
    gen_moves(board, GenMode::All, &mut moves);
    moves
}

/// Generate all legal moves into the engine's fixed-capacity move list.
pub fn generate_legal_movelist(board: &Board) -> MoveList {
    let mut moves = MoveList::new();
    gen_moves(board, GenMode::All, &mut moves);
    moves
}

/// Generate only legal quiet moves.
pub fn generate_quiets(board: &Board) -> MoveList {
    let mut moves = MoveList::new();
    gen_moves(board, GenMode::Quiets, &mut moves);
    moves
}

/// Generate only legal captures (for quiescence search).
pub fn generate_captures(board: &mut Board) -> MoveList {
    let mut moves = MoveList::new();
    let us = board.side_to_move;
    let them = !us;

    if !has_pseudo_capture(board, us, them) {
        return moves;
    }

    let king_sq = board.king_sq(us);
    let pinned = compute_pinned(board, king_sq, us, them);

    if pinned.is_empty() {
        gen_unpinned_captures(board, us, them, king_sq, board.checkers(), &mut moves);
    } else {
        gen_moves(board, GenMode::Captures, &mut moves);
    }

    moves
}

fn has_pseudo_capture(board: &Board, us: Color, them: Color) -> bool {
    let atk = &*ATTACKS;
    let their_occ = board.color_occ(them) & !board.pieces(them, Piece::King);

    if (atk.king(board.king_sq(us)) & their_occ).any() {
        return true;
    }

    let pawns = board.pieces(us, Piece::Pawn);
    let pawn_attacks = if us == Color::White {
        pawns.north_east() | pawns.north_west()
    } else {
        pawns.south_east() | pawns.south_west()
    };
    if (pawn_attacks & their_occ).any() {
        return true;
    }
    if let Some(ep_sq) = board.ep_square() {
        let ep_attackers = atk.pawn(them, ep_sq) & pawns;
        if ep_attackers.any() {
            return true;
        }
    }

    let mut knights = board.pieces(us, Piece::Knight);
    while knights.any() {
        if (atk.knight(knights.pop_lsb()) & their_occ).any() {
            return true;
        }
    }

    let all_occ = board.all_occ;
    let mut bishops = board.pieces(us, Piece::Bishop);
    while bishops.any() {
        if (atk.bishop(bishops.pop_lsb(), all_occ) & their_occ).any() {
            return true;
        }
    }

    let mut rooks = board.pieces(us, Piece::Rook);
    while rooks.any() {
        if (atk.rook(rooks.pop_lsb(), all_occ) & their_occ).any() {
            return true;
        }
    }

    let mut queens = board.pieces(us, Piece::Queen);
    while queens.any() {
        if (atk.queen(queens.pop_lsb(), all_occ) & their_occ).any() {
            return true;
        }
    }

    false
}

/// Recursive perft — counts leaf nodes at depth `depth`.
pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = generate_legal_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut nodes = 0u64;
    for mv in moves {
        board.make_move(mv);
        nodes += perft(board, depth - 1);
        board.unmake_move(mv);
    }
    nodes
}

// -----------------------------------------------------------------------
// Core generation
// -----------------------------------------------------------------------

trait MoveSink {
    fn push_move(&mut self, mv: Move);
}

impl MoveSink for Vec<Move> {
    #[inline(always)]
    fn push_move(&mut self, mv: Move) {
        self.push(mv);
    }
}

impl MoveSink for MoveList {
    #[inline(always)]
    fn push_move(&mut self, mv: Move) {
        self.push(mv);
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum GenMode {
    All,
    Captures,
    Quiets,
}

impl GenMode {
    #[inline(always)]
    fn includes_captures(self) -> bool {
        self != Self::Quiets
    }

    #[inline(always)]
    fn includes_quiets(self) -> bool {
        self != Self::Captures
    }
}

fn gen_moves<S: MoveSink>(board: &Board, mode: GenMode, moves: &mut S) {
    let us = board.side_to_move;
    let them = !us;
    let atk = &*ATTACKS;

    let our_occ = board.color_occ(us);
    let their_occ = board.color_occ(them);
    let all_occ = board.all_occ;

    let king_sq = board.king_sq(us);

    // Cached by Board; refreshed lazily after make/unmake.
    let checkers = board.checkers();
    let in_double_check = checkers.more_than_one();

    // Compute pinned pieces (sliders aligned with our king on both sides)
    let pinned = compute_pinned(board, king_sq, us, them);

    // --- King moves (always generated) ---
    let king_targets = atk.king(king_sq) & !our_occ;
    let targets = match mode {
        GenMode::All => king_targets,
        GenMode::Captures => king_targets & their_occ,
        GenMode::Quiets => king_targets & !their_occ,
    };
    for to in targets {
        // Make sure the destination isn't attacked (use occ without the king)
        let occ_no_king = all_occ ^ Bitboard::from(king_sq);
        if !board.is_attacked_with_occ(to, them, occ_no_king) {
            add_move(king_sq, to, their_occ, moves);
        }
    }

    // In double check only king moves are legal
    if in_double_check {
        return;
    }

    // Compute mask of target squares for non-king pieces.
    // In single check: must block or capture the checker.
    let check_mask = if checkers.any() {
        let checker_sq = checkers.lsb();
        // Captures of the checker + squares between king and checker
        Bitboard::from(checker_sq) | between(king_sq, checker_sq)
    } else {
        Bitboard::FULL // no check: any target is fine
    };

    // --- Pawns ---
    gen_pawn_moves(
        board, us, them, their_occ, all_occ, pinned, king_sq, check_mask, mode, moves,
    );

    // --- Knights ---
    let mut knights = board.pieces(us, Piece::Knight) & !pinned;
    while knights.any() {
        let from = knights.pop_lsb();
        let raw = atk.knight(from) & !our_occ & check_mask;
        let targets = match mode {
            GenMode::All => raw,
            GenMode::Captures => raw & their_occ,
            GenMode::Quiets => raw & !their_occ,
        };
        for to in targets {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Bishops ---
    let mut bishops = board.pieces(us, Piece::Bishop);
    while bishops.any() {
        let from = bishops.pop_lsb();
        let raw = atk.bishop(from, all_occ) & !our_occ & check_mask;
        let targets = match mode {
            GenMode::All => raw,
            GenMode::Captures => raw & their_occ,
            GenMode::Quiets => raw & !their_occ,
        };
        for to in filter_pinned(from, targets, pinned, king_sq) {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Rooks ---
    let mut rooks = board.pieces(us, Piece::Rook);
    while rooks.any() {
        let from = rooks.pop_lsb();
        let raw = atk.rook(from, all_occ) & !our_occ & check_mask;
        let targets = match mode {
            GenMode::All => raw,
            GenMode::Captures => raw & their_occ,
            GenMode::Quiets => raw & !their_occ,
        };
        for to in filter_pinned(from, targets, pinned, king_sq) {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Queens ---
    let mut queens = board.pieces(us, Piece::Queen);
    while queens.any() {
        let from = queens.pop_lsb();
        let raw = atk.queen(from, all_occ) & !our_occ & check_mask;
        let targets = match mode {
            GenMode::All => raw,
            GenMode::Captures => raw & their_occ,
            GenMode::Quiets => raw & !their_occ,
        };
        for to in filter_pinned(from, targets, pinned, king_sq) {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Castling (only when not in check, not captures_only) ---
    if mode.includes_quiets() && !checkers.any() {
        gen_castling(board, us, them, all_occ, moves);
    }
}

fn gen_unpinned_captures(
    board: &Board,
    us: Color,
    them: Color,
    king_sq: Square,
    checkers: Bitboard,
    moves: &mut MoveList,
) {
    let atk = &*ATTACKS;
    let their_occ = board.color_occ(them) & !board.pieces(them, Piece::King);
    let all_occ = board.all_occ;
    let king_bb = Bitboard::from(king_sq);

    for to in atk.king(king_sq) & their_occ {
        if !board.is_attacked_with_occ(to, them, all_occ ^ king_bb) {
            moves.push_move(Move::new(king_sq, to, CAPTURE));
        }
    }

    if checkers.more_than_one() {
        return;
    }

    let target_mask = if checkers.any() { checkers } else { their_occ };

    let mut pawns = board.pieces(us, Piece::Pawn);
    while pawns.any() {
        let from = pawns.pop_lsb();
        for to in atk.pawn(us, from) & target_mask {
            push_pawn_move_flags(from, to, true, moves);
        }

        if let Some(ep_sq) = board.ep_square() {
            if (atk.pawn(us, from) & Bitboard::from(ep_sq)).any() {
                let ep_cap_sq = if us == Color::White {
                    Square(ep_sq.0 - 8)
                } else {
                    Square(ep_sq.0 + 8)
                };
                let captures_checker =
                    checkers.is_empty() || (checkers & Bitboard::from(ep_cap_sq)).any();

                if captures_checker && ep_capture_is_legal(board, us, them, from, ep_sq, ep_cap_sq)
                {
                    moves.push_move(Move::new(from, ep_sq, EN_PASSANT));
                }
            }
        }
    }

    let mut knights = board.pieces(us, Piece::Knight);
    while knights.any() {
        let from = knights.pop_lsb();
        for to in atk.knight(from) & target_mask {
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }

    let mut bishops = board.pieces(us, Piece::Bishop);
    while bishops.any() {
        let from = bishops.pop_lsb();
        for to in atk.bishop(from, all_occ) & target_mask {
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }

    let mut rooks = board.pieces(us, Piece::Rook);
    while rooks.any() {
        let from = rooks.pop_lsb();
        for to in atk.rook(from, all_occ) & target_mask {
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }

    let mut queens = board.pieces(us, Piece::Queen);
    while queens.any() {
        let from = queens.pop_lsb();
        for to in atk.queen(from, all_occ) & target_mask {
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }
}

#[inline]
fn ep_capture_is_legal(
    board: &Board,
    us: Color,
    them: Color,
    from: Square,
    ep_sq: Square,
    ep_cap_sq: Square,
) -> bool {
    let atk = &*ATTACKS;
    let king_sq = board.king_sq(us);
    let occ_after =
        board.all_occ ^ Bitboard::from(from) ^ Bitboard::from(ep_sq) ^ Bitboard::from(ep_cap_sq);
    let exposed_rook = (board.pieces(them, Piece::Rook) | board.pieces(them, Piece::Queen))
        & atk.rook(king_sq, occ_after);
    let exposed_diag = (board.pieces(them, Piece::Bishop) | board.pieces(them, Piece::Queen))
        & atk.bishop(king_sq, occ_after);

    exposed_rook.is_empty() && exposed_diag.is_empty()
}

// -----------------------------------------------------------------------
// Pawn move generation
// -----------------------------------------------------------------------

fn gen_pawn_moves<S: MoveSink>(
    board: &Board,
    us: Color,
    them: Color,
    their_occ: Bitboard,
    all_occ: Bitboard,
    pinned: Bitboard,
    king_sq: Square,
    check_mask: Bitboard,
    mode: GenMode,
    moves: &mut S,
) {
    let atk = &*ATTACKS;
    let pawns = board.pieces(us, Piece::Pawn);

    let rank2: Bitboard = match us {
        Color::White => Bitboard::RANK_2,
        Color::Black => Bitboard::RANK_7,
    };

    let push_one = |bb: Bitboard| -> Bitboard {
        match us {
            Color::White => bb.north(),
            Color::Black => bb.south(),
        }
    };

    let mut pawn_bb = pawns;
    while pawn_bb.any() {
        let from = pawn_bb.pop_lsb();
        let from_bb = Bitboard::from(from);
        let is_pinned = (pinned & from_bb).any();

        // --- Quiet pushes ---
        if mode.includes_quiets() {
            let single_dest = push_one(from_bb) & !all_occ;
            if single_dest.any() {
                let single_sq = single_dest.lsb();
                let in_mask = (single_dest & check_mask).any();
                if in_mask && !(is_pinned && !on_same_ray(from, single_sq, king_sq)) {
                    push_pawn_move_flags(from, single_sq, false, moves);
                }

                // Double push from starting rank
                if (from_bb & rank2).any() {
                    let double_dest = push_one(single_dest) & !all_occ & check_mask;
                    if double_dest.any() {
                        let to = double_dest.lsb();
                        if !(is_pinned && !on_same_ray(from, to, king_sq)) {
                            moves.push_move(Move::new(from, to, DOUBLE_PUSH));
                        }
                    }
                }
            }
        }

        // --- Captures ---
        if mode.includes_captures() {
            let capture_targets = atk.pawn(us, from) & their_occ & check_mask;
            for to in capture_targets {
                if is_pinned && !on_same_ray(from, to, king_sq) {
                    continue;
                }
                push_pawn_move_flags(from, to, true, moves);
            }
        }

        // --- En passant ---
        if mode.includes_captures()
            && let Some(ep_sq) = board.ep_square()
        {
            if (atk.pawn(us, from) & Bitboard::from(ep_sq)).any() {
                let ep_cap_sq = if us == Color::White {
                    Square(ep_sq.0 - 8)
                } else {
                    Square(ep_sq.0 + 8)
                };

                // Verify ep doesn't expose king to horizontal slider
                let occ_after = all_occ
                    ^ Bitboard::from(from)
                    ^ Bitboard::from(ep_sq)
                    ^ Bitboard::from(ep_cap_sq);
                let exposed_rook = (board.pieces(them, Piece::Rook)
                    | board.pieces(them, Piece::Queen))
                    & atk.rook(king_sq, occ_after);
                let exposed_diag = (board.pieces(them, Piece::Bishop)
                    | board.pieces(them, Piece::Queen))
                    & atk.bishop(king_sq, occ_after);

                // EP is only valid when it resolves a check (or there's no check)
                let ep_resolves = (check_mask & Bitboard::from(ep_sq)).any()
                    || (check_mask & Bitboard::from(ep_cap_sq)).any()
                    || check_mask.0 == u64::MAX;

                if ep_resolves && exposed_rook.is_empty() && exposed_diag.is_empty() {
                    moves.push_move(Move::new(from, ep_sq, EN_PASSANT));
                }
            }
        }
    }
}

/// Emit pawn move(s) — either simple or promotion set.
#[inline]
fn push_pawn_move_flags<S: MoveSink>(from: Square, to: Square, is_capture: bool, moves: &mut S) {
    let is_promo = to.rank() == Rank::R8 || to.rank() == Rank::R1;
    if is_promo {
        if is_capture {
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_QUEEN));
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_ROOK));
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_BISHOP));
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_KNIGHT));
        } else {
            moves.push_move(Move::new(from, to, PROMO_QUEEN));
            moves.push_move(Move::new(from, to, PROMO_ROOK));
            moves.push_move(Move::new(from, to, PROMO_BISHOP));
            moves.push_move(Move::new(from, to, PROMO_KNIGHT));
        }
    } else if is_capture {
        moves.push_move(Move::new(from, to, CAPTURE));
    } else {
        moves.push_move(Move::new(from, to, QUIET));
    }
}

/// Add a single non-pawn move (capture or quiet).
#[inline(always)]
fn add_move<S: MoveSink>(from: Square, to: Square, their_occ: Bitboard, moves: &mut S) {
    if (Bitboard::from(to) & their_occ).any() {
        moves.push_move(Move::new(from, to, CAPTURE));
    } else {
        moves.push_move(Move::new(from, to, QUIET));
    }
}

// -----------------------------------------------------------------------
// Castling
// -----------------------------------------------------------------------

fn gen_castling<S: MoveSink>(
    board: &Board,
    us: Color,
    them: Color,
    all_occ: Bitboard,
    moves: &mut S,
) {
    let (ks_flag, qs_flag, king_sq, ks_rook, qs_rook, ks_empty, qs_empty, ks_safe, qs_safe) = if us
        == Color::White
    {
        (
            CastlingRights::WHITE_KINGSIDE,
            CastlingRights::WHITE_QUEENSIDE,
            Square::E1,
            Square::H1,
            Square::A1,
            // Squares that must be empty for KS / QS
            Bitboard::from(Square::F1) | Bitboard::from(Square::G1),
            Bitboard::from(Square::B1) | Bitboard::from(Square::C1) | Bitboard::from(Square::D1),
            // Squares that must not be attacked for KS / QS (king path)
            [Square::F1, Square::G1],
            [Square::C1, Square::D1],
        )
    } else {
        (
            CastlingRights::BLACK_KINGSIDE,
            CastlingRights::BLACK_QUEENSIDE,
            Square::E8,
            Square::H8,
            Square::A8,
            Bitboard::from(Square::F8) | Bitboard::from(Square::G8),
            Bitboard::from(Square::B8) | Bitboard::from(Square::C8) | Bitboard::from(Square::D8),
            [Square::F8, Square::G8],
            [Square::C8, Square::D8],
        )
    };

    // Verify the rook is actually present (handles FEN edge cases)
    if board.castling.has(ks_flag)
        && (all_occ & ks_empty).is_empty()
        && (board.pieces(us, Piece::Rook) & Bitboard::from(ks_rook)).any()
        && ks_safe.iter().all(|&sq| !board.is_attacked(sq, them))
    {
        moves.push_move(Move::new(king_sq, ks_safe[1], CASTLE_KINGSIDE));
    }

    if board.castling.has(qs_flag)
        && (all_occ & qs_empty).is_empty()
        && (board.pieces(us, Piece::Rook) & Bitboard::from(qs_rook)).any()
        && qs_safe.iter().all(|&sq| !board.is_attacked(sq, them))
    {
        moves.push_move(Move::new(king_sq, qs_safe[0], CASTLE_QUEENSIDE));
    }
}

// -----------------------------------------------------------------------
// Pin detection
// -----------------------------------------------------------------------

/// Compute the bitboard of our pieces that are pinned to our king.
fn compute_pinned(board: &Board, king_sq: Square, us: Color, them: Color) -> Bitboard {
    let our_occ = board.color_occ(us);
    let atk = &*ATTACKS;
    let mut pinned = Bitboard::EMPTY;

    // X-ray diagonal: see through our own pieces to find diagonal pinners
    let bishop_vision = atk.bishop(king_sq, board.all_occ);
    let xray_bishop = atk.bishop(king_sq, board.all_occ ^ (bishop_vision & our_occ));
    let diag_pinners =
        (board.pieces(them, Piece::Bishop) | board.pieces(them, Piece::Queen)) & xray_bishop;
    for pinner_sq in diag_pinners {
        let ray = between(king_sq, pinner_sq);
        let blockers = ray & our_occ;
        if blockers.count() == 1 {
            pinned |= blockers;
        }
    }

    // X-ray orthogonal: see through our own pieces to find orthogonal pinners
    let rook_vision = atk.rook(king_sq, board.all_occ);
    let xray_rook = atk.rook(king_sq, board.all_occ ^ (rook_vision & our_occ));
    let ortho_pinners =
        (board.pieces(them, Piece::Rook) | board.pieces(them, Piece::Queen)) & xray_rook;
    for pinner_sq in ortho_pinners {
        let ray = between(king_sq, pinner_sq);
        let blockers = ray & our_occ;
        if blockers.count() == 1 {
            pinned |= blockers;
        }
    }

    pinned
}

/// Filter target squares for a pinned piece — it may only move along the pin ray.
#[inline]
fn filter_pinned(from: Square, targets: Bitboard, pinned: Bitboard, king_sq: Square) -> Bitboard {
    if (pinned & Bitboard::from(from)).any() {
        targets & ray_through(from, king_sq)
    } else {
        targets
    }
}

// -----------------------------------------------------------------------
// Geometry helpers
// -----------------------------------------------------------------------

/// Bitboard of squares strictly between `a` and `b` on a rank, file, or diagonal.
/// Returns `EMPTY` if they are not aligned.
pub fn between(a: Square, b: Square) -> Bitboard {
    // Use precomputed table
    BETWEEN[a.index()][b.index()]
}

/// Full ray through both `a` and `b` (including both endpoints).
pub fn ray_through(a: Square, b: Square) -> Bitboard {
    LINE[a.index()][b.index()]
}

/// Returns true if `from`, `to`, and `king` are all on the same rank/file/diagonal.
#[inline]
fn on_same_ray(from: Square, to: Square, king: Square) -> bool {
    (ray_through(from, to) & Bitboard::from(king)).any()
}

// -----------------------------------------------------------------------
// Precomputed between / line tables
// -----------------------------------------------------------------------

static BETWEEN: LazyLock<[[Bitboard; 64]; 64]> = LazyLock::new(init_between);
static LINE: LazyLock<[[Bitboard; 64]; 64]> = LazyLock::new(init_line);

fn init_between() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];
    for a in 0..64u8 {
        for b in 0..64u8 {
            if a == b {
                continue;
            }
            let ar = a / 8;
            let af = a % 8;
            let br = b / 8;
            let bf = b % 8;
            let dr = br as i8 - ar as i8;
            let df = bf as i8 - af as i8;

            // Check alignment
            if ar == br || af == bf || dr.abs() == df.abs() {
                let sr: i8 = dr.signum();
                let sf: i8 = df.signum();
                let mut r = ar as i8 + sr;
                let mut f = af as i8 + sf;
                while r != br as i8 || f != bf as i8 {
                    table[a as usize][b as usize] |=
                        Bitboard::from(Square((r as u8) * 8 + f as u8));
                    r += sr;
                    f += sf;
                }
            }
        }
    }
    table
}

fn init_line() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];
    for a in 0..64u8 {
        for b in 0..64u8 {
            if a == b {
                continue;
            }
            let ar = a / 8;
            let af = a % 8;
            let br = b / 8;
            let bf = b % 8;
            let dr = br as i8 - ar as i8;
            let df = bf as i8 - af as i8;

            if ar == br || af == bf || dr.abs() == df.abs() {
                let sr: i8 = dr.signum();
                let sf: i8 = df.signum();

                // Extend the ray in both directions from `a`
                let mut bits = Bitboard::from(Square(a)) | Bitboard::from(Square(b));

                // Forward from a
                let (mut r, mut f) = (ar as i8 + sr, af as i8 + sf);
                while (0..8).contains(&r) && (0..8).contains(&f) {
                    bits |= Bitboard::from(Square(r as u8 * 8 + f as u8));
                    r += sr;
                    f += sf;
                }
                // Backward from a
                let (mut r, mut f) = (ar as i8 - sr, af as i8 - sf);
                while (0..8).contains(&r) && (0..8).contains(&f) {
                    bits |= Bitboard::from(Square(r as u8 * 8 + f as u8));
                    r -= sr;
                    f -= sf;
                }

                table[a as usize][b as usize] = bits;
            }
        }
    }
    table
}

// -----------------------------------------------------------------------
// Extend Board with occ-parameterized attack check (needed for EP)
// -----------------------------------------------------------------------

impl Board {
    pub fn is_attacked_with_occ(&self, sq: Square, attacker: Color, occ: Bitboard) -> bool {
        let atk = &*ATTACKS;
        if (atk.pawn(!attacker, sq) & self.pieces(attacker, Piece::Pawn)).any() {
            return true;
        }
        if (atk.knight(sq) & self.pieces(attacker, Piece::Knight)).any() {
            return true;
        }
        if (atk.king(sq) & self.pieces(attacker, Piece::King)).any() {
            return true;
        }
        if (atk.bishop(sq, occ)
            & (self.pieces(attacker, Piece::Bishop) | self.pieces(attacker, Piece::Queen)))
        .any()
        {
            return true;
        }
        if (atk.rook(sq, occ)
            & (self.pieces(attacker, Piece::Rook) | self.pieces(attacker, Piece::Queen)))
        .any()
        {
            return true;
        }
        false
    }
}
