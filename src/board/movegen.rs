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
    Move, MoveList, CAPTURE, CASTLE_KINGSIDE, CASTLE_QUEENSIDE, DOUBLE_PUSH, EN_PASSANT,
    PROMO_CAPTURE_BISHOP, PROMO_CAPTURE_KNIGHT, PROMO_CAPTURE_QUEEN, PROMO_CAPTURE_ROOK,
    PROMO_BISHOP, PROMO_KNIGHT, PROMO_QUEEN, PROMO_ROOK, QUIET,
};
use super::piece::{CastlingRights, Color, Piece};
use super::square::{Rank, Square};

// -----------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------

/// Generate all legal moves for the current position.
pub fn generate_legal_moves(board: &Board) -> MoveList {
    let mut moves = MoveList::new();
    gen_moves(board, false, &mut moves);
    moves
}

/// Generate only captures and promotions (for quiescence search).
pub fn generate_captures(board: &Board) -> MoveList {
    let mut moves = MoveList::new();
    gen_moves(board, true, &mut moves);
    moves
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
// MVV-LVA capture scoring
// -----------------------------------------------------------------------

/// MVV-LVA (Most Valuable Victim − Least Valuable Attacker) score for a
/// capture move.  Higher score → search earlier.
///
/// Designed for capture move ordering in alpha-beta: a queen capturing a
/// pawn ranks lower than a pawn capturing a queen.
#[inline(always)]
pub fn mvv_lva(board: &Board, mv: Move) -> i16 {
    // Victim value (the piece being taken)
    const VICTIM: [i16; 6]   = [10, 30, 30, 50, 90, 200]; // P N B R Q K
    // Attacker value (subtracted, so cheaper attackers rank higher)
    const ATTACKER: [i16; 6] = [ 1,  3,  3,  5,  9,  99]; // P N B R Q K

    let victim = if mv.is_en_passant() {
        Piece::Pawn
    } else {
        board.piece_type_at(mv.to_sq()).unwrap_or(Piece::Pawn)
    };
    let attacker = board.piece_type_at(mv.from_sq()).unwrap_or(Piece::Pawn);

    VICTIM[victim as usize] - ATTACKER[attacker as usize]
}

// -----------------------------------------------------------------------
// Core generation
// -----------------------------------------------------------------------

fn gen_moves(board: &Board, captures_only: bool, moves: &mut MoveList) {
    let us = board.side_to_move;
    let them = !us;
    let atk = &*ATTACKS;

    let our_occ = board.color_occ(us);
    let their_occ = board.color_occ(them);
    let all_occ = board.all_occ;

    let king_sq = board.king_sq(us);

    // Compute all pieces attacking our king
    let checkers = board.attackers_to(king_sq, all_occ) & their_occ;
    let in_double_check = checkers.more_than_one();

    // Compute pinned pieces (sliders aligned with our king on both sides)
    let pinned = compute_pinned(board, king_sq, us, them);

    // --- King moves (always generated) ---
    let king_targets = atk.king(king_sq) & !our_occ;
    let targets = if captures_only {
        king_targets & their_occ
    } else {
        king_targets
    };
    let occ_no_king = all_occ ^ Bitboard::from(king_sq);
    for to in targets {
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
    gen_pawn_moves(board, us, them, their_occ, all_occ, pinned, king_sq, check_mask, captures_only, moves);

    // --- Knights ---
    let mut knights = board.pieces(us, Piece::Knight) & !pinned;
    while knights.any() {
        let from = knights.pop_lsb();
        let raw = atk.knight(from) & !our_occ & check_mask;
        let targets = if captures_only { raw & their_occ } else { raw };
        for to in targets {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Bishops ---
    let mut bishops = board.pieces(us, Piece::Bishop);
    while bishops.any() {
        let from = bishops.pop_lsb();
        let raw = atk.bishop(from, all_occ) & !our_occ & check_mask;
        let targets = if captures_only { raw & their_occ } else { raw };
        for to in filter_pinned(from, targets, pinned, king_sq) {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Rooks ---
    let mut rooks = board.pieces(us, Piece::Rook);
    while rooks.any() {
        let from = rooks.pop_lsb();
        let raw = atk.rook(from, all_occ) & !our_occ & check_mask;
        let targets = if captures_only { raw & their_occ } else { raw };
        for to in filter_pinned(from, targets, pinned, king_sq) {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Queens ---
    let mut queens = board.pieces(us, Piece::Queen);
    while queens.any() {
        let from = queens.pop_lsb();
        let raw = atk.queen(from, all_occ) & !our_occ & check_mask;
        let targets = if captures_only { raw & their_occ } else { raw };
        for to in filter_pinned(from, targets, pinned, king_sq) {
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Castling (only when not in check, not captures_only) ---
    if !captures_only && !checkers.any() {
        gen_castling(board, us, them, all_occ, moves);
    }
}

// -----------------------------------------------------------------------
// Pawn move generation (bulk bitboard approach)
// -----------------------------------------------------------------------

fn gen_pawn_moves(
    board: &Board,
    us: Color,
    them: Color,
    their_occ: Bitboard,
    all_occ: Bitboard,
    pinned: Bitboard,
    king_sq: Square,
    check_mask: Bitboard,
    captures_only: bool,
    moves: &mut MoveList,
) {
    let atk = &*ATTACKS;
    let pawns = board.pieces(us, Piece::Pawn);
    let free_pawns = pawns & !pinned;
    let pinned_pawns = pawns & pinned;

    let (push_off, cap_w_off, cap_e_off, rank_start, promo_rank) = match us {
        Color::White => (8i32, 7i32, 9i32, Bitboard::RANK_2, Bitboard::RANK_8),
        Color::Black => (-8i32, -9i32, -7i32, Bitboard::RANK_7, Bitboard::RANK_1),
    };

    macro_rules! push_one {
        ($bb:expr) => {
            match us {
                Color::White => ($bb).north(),
                Color::Black => ($bb).south(),
            }
        };
    }
    macro_rules! cap_west {
        ($bb:expr) => {
            match us {
                Color::White => ($bb).north_west(),
                Color::Black => ($bb).south_west(),
            }
        };
    }
    macro_rules! cap_east {
        ($bb:expr) => {
            match us {
                Color::White => ($bb).north_east(),
                Color::Black => ($bb).south_east(),
            }
        };
    }

    // -------------------------------------------------------------------
    // Bulk generation for unpinned pawns
    // -------------------------------------------------------------------

    if !captures_only {
        let singles = push_one!(free_pawns) & !all_occ;
        let valid_singles = singles & check_mask;

        for to in valid_singles & !promo_rank {
            moves.push(Move::new(Square((to.0 as i32 - push_off) as u8), to, QUIET));
        }
        for to in valid_singles & promo_rank {
            let from = Square((to.0 as i32 - push_off) as u8);
            moves.push(Move::new(from, to, PROMO_QUEEN));
            moves.push(Move::new(from, to, PROMO_ROOK));
            moves.push(Move::new(from, to, PROMO_BISHOP));
            moves.push(Move::new(from, to, PROMO_KNIGHT));
        }

        // Double pushes: only from the starting rank, through an empty square
        let can_double = push_one!(free_pawns & rank_start) & !all_occ;
        for to in push_one!(can_double) & !all_occ & check_mask {
            moves.push(Move::new(Square((to.0 as i32 - push_off * 2) as u8), to, DOUBLE_PUSH));
        }
    }

    // West captures
    let west_caps = cap_west!(free_pawns) & their_occ & check_mask;
    for to in west_caps & !promo_rank {
        moves.push(Move::new(Square((to.0 as i32 - cap_w_off) as u8), to, CAPTURE));
    }
    for to in west_caps & promo_rank {
        let from = Square((to.0 as i32 - cap_w_off) as u8);
        moves.push(Move::new(from, to, PROMO_CAPTURE_QUEEN));
        moves.push(Move::new(from, to, PROMO_CAPTURE_ROOK));
        moves.push(Move::new(from, to, PROMO_CAPTURE_BISHOP));
        moves.push(Move::new(from, to, PROMO_CAPTURE_KNIGHT));
    }

    // East captures
    let east_caps = cap_east!(free_pawns) & their_occ & check_mask;
    for to in east_caps & !promo_rank {
        moves.push(Move::new(Square((to.0 as i32 - cap_e_off) as u8), to, CAPTURE));
    }
    for to in east_caps & promo_rank {
        let from = Square((to.0 as i32 - cap_e_off) as u8);
        moves.push(Move::new(from, to, PROMO_CAPTURE_QUEEN));
        moves.push(Move::new(from, to, PROMO_CAPTURE_ROOK));
        moves.push(Move::new(from, to, PROMO_CAPTURE_BISHOP));
        moves.push(Move::new(from, to, PROMO_CAPTURE_KNIGHT));
    }

    // En passant — check which free pawns attack the EP square
    if let Some(ep_sq) = board.ep_square() {
        let ep_bb = Bitboard::from(ep_sq);
        let ep_cap_sq = Square((ep_sq.0 as i32 - push_off) as u8);
        let ep_resolves = (check_mask & ep_bb).any() || (check_mask & Bitboard::from(ep_cap_sq)).any();

        if ep_resolves {
            for from in free_pawns & atk.pawn(them, ep_sq) {
                let occ_after = all_occ ^ Bitboard::from(from) ^ ep_bb ^ Bitboard::from(ep_cap_sq);
                let exposed_rook = (board.pieces(them, Piece::Rook) | board.pieces(them, Piece::Queen))
                    & atk.rook(king_sq, occ_after);
                let exposed_diag = (board.pieces(them, Piece::Bishop) | board.pieces(them, Piece::Queen))
                    & atk.bishop(king_sq, occ_after);
                if exposed_rook.is_empty() && exposed_diag.is_empty() {
                    moves.push(Move::new(from, ep_sq, EN_PASSANT));
                }
            }
        }
    }

    // -------------------------------------------------------------------
    // Per-pawn generation for pinned pawns (must stay on pin ray)
    // -------------------------------------------------------------------

    for from in pinned_pawns {
        let from_bb = Bitboard::from(from);
        let pin_ray = ray_through(from, king_sq);

        if !captures_only {
            let single_dest = push_one!(from_bb) & !all_occ;
            if (single_dest & check_mask & pin_ray).any() {
                let sq = single_dest.lsb();
                push_pawn_move_flags(from, sq, false, moves);

                if (from_bb & rank_start).any() {
                    let double_dest = push_one!(single_dest) & !all_occ & check_mask & pin_ray;
                    if double_dest.any() {
                        moves.push(Move::new(from, double_dest.lsb(), DOUBLE_PUSH));
                    }
                }
            }
        }

        let cap_targets = (cap_west!(from_bb) | cap_east!(from_bb)) & their_occ & check_mask & pin_ray;
        for to in cap_targets {
            push_pawn_move_flags(from, to, true, moves);
        }

        // EP for pinned pawns
        if let Some(ep_sq) = board.ep_square() {
            if (atk.pawn(us, from) & Bitboard::from(ep_sq) & pin_ray).any() {
                let ep_bb = Bitboard::from(ep_sq);
                let ep_cap_sq = Square((ep_sq.0 as i32 - push_off) as u8);
                let ep_resolves = (check_mask & ep_bb).any() || (check_mask & Bitboard::from(ep_cap_sq)).any();
                if ep_resolves {
                    let occ_after = all_occ ^ from_bb ^ ep_bb ^ Bitboard::from(ep_cap_sq);
                    let exposed_rook = (board.pieces(them, Piece::Rook) | board.pieces(them, Piece::Queen))
                        & atk.rook(king_sq, occ_after);
                    let exposed_diag = (board.pieces(them, Piece::Bishop) | board.pieces(them, Piece::Queen))
                        & atk.bishop(king_sq, occ_after);
                    if exposed_rook.is_empty() && exposed_diag.is_empty() {
                        moves.push(Move::new(from, ep_sq, EN_PASSANT));
                    }
                }
            }
        }
    }
}

/// Emit pawn move(s) — either simple or promotion set.
#[inline]
fn push_pawn_move_flags(from: Square, to: Square, is_capture: bool, moves: &mut MoveList) {
    let is_promo = to.rank() == Rank::R8 || to.rank() == Rank::R1;
    if is_promo {
        if is_capture {
            moves.push(Move::new(from, to, PROMO_CAPTURE_QUEEN));
            moves.push(Move::new(from, to, PROMO_CAPTURE_ROOK));
            moves.push(Move::new(from, to, PROMO_CAPTURE_BISHOP));
            moves.push(Move::new(from, to, PROMO_CAPTURE_KNIGHT));
        } else {
            moves.push(Move::new(from, to, PROMO_QUEEN));
            moves.push(Move::new(from, to, PROMO_ROOK));
            moves.push(Move::new(from, to, PROMO_BISHOP));
            moves.push(Move::new(from, to, PROMO_KNIGHT));
        }
    } else if is_capture {
        moves.push(Move::new(from, to, CAPTURE));
    } else {
        moves.push(Move::new(from, to, QUIET));
    }
}

/// Add a single non-pawn move (capture or quiet).
#[inline(always)]
fn add_move(from: Square, to: Square, their_occ: Bitboard, moves: &mut MoveList) {
    if (Bitboard::from(to) & their_occ).any() {
        moves.push(Move::new(from, to, CAPTURE));
    } else {
        moves.push(Move::new(from, to, QUIET));
    }
}

// -----------------------------------------------------------------------
// Castling
// -----------------------------------------------------------------------

fn gen_castling(board: &Board, us: Color, them: Color, all_occ: Bitboard, moves: &mut MoveList) {
    let (ks_flag, qs_flag, king_sq, ks_rook, qs_rook, ks_empty, qs_empty, ks_safe, qs_safe) =
        if us == Color::White {
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
        moves.push(Move::new(king_sq, ks_safe[1], CASTLE_KINGSIDE));
    }

    if board.castling.has(qs_flag)
        && (all_occ & qs_empty).is_empty()
        && (board.pieces(us, Piece::Rook) & Bitboard::from(qs_rook)).any()
        && qs_safe.iter().all(|&sq| !board.is_attacked(sq, them))
    {
        moves.push(Move::new(king_sq, qs_safe[0], CASTLE_QUEENSIDE));
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
    let diag_pinners = (board.pieces(them, Piece::Bishop) | board.pieces(them, Piece::Queen))
        & xray_bishop;
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
    let ortho_pinners = (board.pieces(them, Piece::Rook) | board.pieces(them, Piece::Queen))
        & xray_rook;
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
    BETWEEN[a.index()][b.index()]
}

/// Full ray through both `a` and `b` (including both endpoints).
pub fn ray_through(a: Square, b: Square) -> Bitboard {
    LINE[a.index()][b.index()]
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
                    table[a as usize][b as usize] |= Bitboard::from(Square((r as u8) * 8 + f as u8));
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
        if (atk.bishop(sq, occ) & (self.pieces(attacker, Piece::Bishop) | self.pieces(attacker, Piece::Queen))).any() {
            return true;
        }
        if (atk.rook(sq, occ) & (self.pieces(attacker, Piece::Rook) | self.pieces(attacker, Piece::Queen))).any() {
            return true;
        }
        false
    }
}
