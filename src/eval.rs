//! Static position evaluator - PeSTO PST + structural terms.
//!
//! Returns score in centipawns from the side-to-move's perspective.

use crate::board::attacks::ATTACKS;
use crate::board::bitboard::Bitboard;
use crate::board::board::Board;
use crate::board::piece::{Color, Piece};
use crate::board::square::Square;

const MG_VAL: [i32; 6] = [82, 337, 365, 477, 1025, 0];
const EG_VAL: [i32; 6] = [94, 281, 297, 512, 936, 0];
const PHASE_WEIGHT: [i32; 6] = [0, 1, 1, 2, 4, 0];
const TOTAL_PHASE: i32 = 24;

#[rustfmt::skip]
const MG_PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    98,134, 61, 95, 68,126, 34,-11,
    -6,  7, 26, 31, 65, 56, 25,-20,
   -14, 13,  6, 21, 23, 12, 17,-23,
   -27, -2, -5, 12, 17,  6, 10,-25,
   -26, -4, -4,-10,  3,  3, 33,-12,
   -35, -1,-20,-23,-15, 24, 38,-22,
     0,  0,  0,  0,  0,  0,  0,  0,
];
#[rustfmt::skip]
const EG_PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
   178,173,158,134,147,132,165,187,
    94,100, 85, 67, 56, 53, 82, 84,
    32, 24, 13,  5, -2,  4, 17, 17,
    13,  9, -3, -7, -7, -8,  3, -1,
     4,  7, -6,  1,  0, -5, -1, -8,
    13,  8,  8, 10, 13,  0,  2, -7,
     0,  0,  0,  0,  0,  0,  0,  0,
];
#[rustfmt::skip]
const MG_KNIGHT_TABLE: [i32; 64] = [
  -167,-89,-34,-49, 61,-97,-15,-107,
   -73,-41, 72, 36, 23, 62,  7, -17,
   -47, 60, 37, 65, 84,129, 73,  44,
    -9, 17, 19, 53, 37, 69, 18,  22,
   -13,  4, 16, 13, 28, 19, 21,  -8,
   -23, -9, 12, 10, 19, 17, 25, -16,
   -29,-53,-12, -3, -1, 18,-14, -19,
  -105,-21,-58,-33,-17,-28,-19, -23,
];
#[rustfmt::skip]
const EG_KNIGHT_TABLE: [i32; 64] = [
   -58,-38,-13,-28,-31,-27,-63,-99,
   -25, -8,-25, -2, -9,-25,-24,-52,
   -24,-20, 10,  9, -1, -9,-19,-41,
   -17,  3, 22, 22, 22, 11,  8,-18,
   -18, -6, 16, 25, 16, 17,  4,-18,
   -23, -3, -1, 15, 10, -3,-20,-22,
   -42,-20,-10, -5, -2,-20,-23,-44,
   -29,-51,-23,-15,-22,-18,-50,-64,
];
#[rustfmt::skip]
const MG_BISHOP_TABLE: [i32; 64] = [
   -29,  4,-82,-37,-25,-42,  7, -8,
   -26, 16,-18,-13, 30, 59, 18,-47,
   -16, 37, 43, 40, 35, 50, 37, -2,
    -4,  5, 19, 50, 37, 37,  7, -2,
    -6, 13, 13, 26, 34, 12, 10,  4,
     0, 15, 15, 15, 14, 27, 16,  0,
     4, 15, 16,  0,  7, 21, 33,  1,
   -33, -3,-14,-21,-13,-12,-39,-21,
];
#[rustfmt::skip]
const EG_BISHOP_TABLE: [i32; 64] = [
   -14,-21,-11, -8, -7, -9,-17,-24,
    -8, -4,  7,-12, -3,-13, -4,-14,
     2, -8,  0, -1, -2,  6,  0,  4,
    -3,  9, 12,  9, 14, 10,  3,  2,
    -6,  3, 13, 19,  7, 10, -3, -9,
   -12, -3,  8, 10, 13,  3, -7,-15,
   -14,-18, -7, -1,  4, -9,-15,-27,
   -23, -9,-23, -5, -9,-16, -5,-17,
];
#[rustfmt::skip]
const MG_ROOK_TABLE: [i32; 64] = [
    32, 42, 32, 51, 63,  9, 31, 43,
    27, 32, 58, 62, 80, 67, 26, 44,
    -5, 19, 26, 36, 17, 45, 61, 16,
   -24,-11,  7, 26, 24, 35, -8,-20,
   -36,-26,-12, -1,  9, -7,  6,-23,
   -45,-25,-16,-17,  3,  0, -5,-33,
   -44,-16,-20, -9, -1, 11, -6,-71,
   -19,-13,  1, 17, 16,  7,-37,-26,
];
#[rustfmt::skip]
const EG_ROOK_TABLE: [i32; 64] = [
    13, 10, 18, 15, 12, 12,  8,  5,
    11, 13, 13, 11, -3,  3,  8,  3,
     7,  7,  7,  5,  4, -3, -5, -3,
     4,  3, 13,  1,  2,  1, -1,  2,
     3,  5,  8,  4, -5, -6, -8, -11,
    -4,  0, -5, -1, -7,-12, -8,-16,
    -6, -6,  0,  2, -9, -9,-11, -3,
    -9,  2,  3, -1, -5,-13,  4,-20,
];
#[rustfmt::skip]
const MG_QUEEN_TABLE: [i32; 64] = [
   -28,  0, 29, 12, 59, 44, 43, 45,
   -24,-39, -5,  1,-16, 57, 28, 54,
   -13,-17,  7,  8, 29, 56, 47, 57,
   -27,-27,-16,-16, -1, 17, -2,  1,
    -9,-26, -9,-10, -2, -4,  3, -3,
   -14,  2,-11, -2, -5,  2, 14,  5,
   -35, -8, 11,  2,  8, 15, -3,  1,
    -1,-18, -9, 10,-15,-25,-31,-50,
];
#[rustfmt::skip]
const EG_QUEEN_TABLE: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20,
   -17, 20, 32, 41, 58, 25, 30,  0,
   -20,  6,  9, 49, 47, 35, 19,  9,
     3, 22, 24, 45, 57, 40, 57, 36,
   -18, 28, 19, 47, 31, 34, 39, 23,
   -16,-27, 15,  6,  9, 17, 10,  5,
   -22,-23,-30,-16,-16,-23,-36,-32,
   -33,-28,-22,-43, -5,-32,-20,-41,
];
#[rustfmt::skip]
const MG_KING_TABLE: [i32; 64] = [
   -65, 23, 16,-15,-56,-34,  2, 13,
    29, -1,-20, -7, -8, -4,-38,-29,
    -9, 24,  2,-16,-20,  6, 22,-22,
   -17,-20,-12,-27,-30,-25,-14,-36,
   -49, -1,-27,-39,-46,-44,-33,-51,
   -14,-14,-22,-46,-44,-30,-15,-27,
     1,  7, -8,-64,-43,-16,  9,  8,
   -15, 36, 12,-54,  8,-28, 24, 14,
];
#[rustfmt::skip]
const EG_KING_TABLE: [i32; 64] = [
   -74,-35,-18,-18,-11, 15,  4,-17,
   -12, 17, 14, 17, 17, 38, 23, 11,
    10, 17, 23, 15, 20, 45, 44, 13,
    -8, 22, 24, 27, 26, 33, 26,  3,
   -18, -4, 21, 24, 27, 23,  9,-11,
   -19, -3, 11, 21, 23, 16,  7, -9,
   -27,-11,  4, 13, 14,  4, -5,-17,
   -53,-34,-21,-11,-28,-14,-24,-43,
];

const MG_TABLE: [[i32; 64]; 6] = [
    MG_PAWN_TABLE,
    MG_KNIGHT_TABLE,
    MG_BISHOP_TABLE,
    MG_ROOK_TABLE,
    MG_QUEEN_TABLE,
    MG_KING_TABLE,
];
const EG_TABLE: [[i32; 64]; 6] = [
    EG_PAWN_TABLE,
    EG_KNIGHT_TABLE,
    EG_BISHOP_TABLE,
    EG_ROOK_TABLE,
    EG_QUEEN_TABLE,
    EG_KING_TABLE,
];

const PASSED_MG: [i32; 8] = [0, 10, 10, 20, 40, 60, 100, 0];
const PASSED_EG: [i32; 8] = [0, 15, 15, 30, 55, 80, 120, 0];
const SAFETY_TABLE: [i32; 16] = [
    0, 0, 10, 25, 40, 60, 80, 95, 105, 110, 112, 114, 115, 116, 117, 118,
];

#[derive(Copy, Clone, Debug)]
struct PawnEntry {
    key: u64,
    mg: i32,
    eg: i32,
    passers: [Bitboard; 2],
}

const PAWN_CACHE_SIZE: usize = 16384;

pub struct Evaluator {
    pawn_cache: Box<[PawnEntry; PAWN_CACHE_SIZE]>,
}

impl Evaluator {
    pub fn new() -> Self {
        const EMPTY_ENTRY: PawnEntry = PawnEntry {
            key: u64::MAX,
            mg: 0,
            eg: 0,
            passers: [Bitboard::EMPTY; 2],
        };
        Self {
            pawn_cache: vec![EMPTY_ENTRY; PAWN_CACHE_SIZE]
                .into_boxed_slice()
                .try_into()
                .expect("pawn cache size"),
        }
    }

    pub fn evaluate(&mut self, board: &Board) -> i32 {
        let (mg, eg, phase) = self.compute(board);
        let phase = phase.min(TOTAL_PHASE);
        let tapered = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
        let mut score = tapered + 10;

        // Draw scaling: opposite-color bishops
        {
            let wb = board.pieces(Color::White, Piece::Bishop);
            let bb = board.pieces(Color::Black, Piece::Bishop);
            let wb1 = wb.any() && !wb.more_than_one();
            let bb1 = bb.any() && !bb.more_than_one();
            if wb1 && bb1 {
                let wb_dark = (wb & Bitboard::DARK_SQUARES).any();
                let bb_dark = (bb & Bitboard::DARK_SQUARES).any();
                if wb_dark != bb_dark {
                    let total_pawns = (board.pieces(Color::White, Piece::Pawn)
                        | board.pieces(Color::Black, Piece::Pawn))
                        .count() as i32;
                    let scale = (32 + total_pawns * 4).min(48);
                    score = score * scale / 48;
                }
            }
        }
        // Two knights vs bare king is a theoretical draw
        {
            let only_king_w = board.color_occ(Color::White)
                == Bitboard::from(board.king_sq(Color::White));
            let only_king_b = board.color_occ(Color::Black)
                == Bitboard::from(board.king_sq(Color::Black));
            let has_only_2n = |c: Color| {
                board.pieces(c, Piece::Pawn).is_empty()
                    && board.pieces(c, Piece::Bishop).is_empty()
                    && board.pieces(c, Piece::Rook).is_empty()
                    && board.pieces(c, Piece::Queen).is_empty()
                    && board.pieces(c, Piece::Knight).count() == 2
            };
            if only_king_w && has_only_2n(Color::Black) {
                score = 0;
            }
            if only_king_b && has_only_2n(Color::White) {
                score = 0;
            }
        }

        if board.side_to_move == Color::White {
            score
        } else {
            -score
        }
    }

    fn compute(&mut self, board: &Board) -> (i32, i32, i32) {
        let mut mg = [0i32; 2];
        let mut eg = [0i32; 2];
        let mut phase = 0i32;
        let atk = &*ATTACKS;

        for color in [Color::White, Color::Black] {
            let c = color as usize;
            let opp = !color;

            for &piece in &Piece::ALL {
                let mut bb = board.pieces(color, piece);
                while bb.any() {
                    let sq = bb.pop_lsb();
                    let pst_sq = if color == Color::White {
                        sq.index()
                    } else {
                        sq.index() ^ 56
                    };
                    mg[c] += MG_VAL[piece as usize] + MG_TABLE[piece as usize][pst_sq];
                    eg[c] += EG_VAL[piece as usize] + EG_TABLE[piece as usize][pst_sq];
                    phase += PHASE_WEIGHT[piece as usize];
                }
            }

            let own_pawns = board.pieces(color, Piece::Pawn);
            let opp_pawns = board.pieces(opp, Piece::Pawn);

            if board.pieces(color, Piece::Bishop).count() >= 2 {
                mg[c] += 30;
                eg[c] += 50;
            }

            let mut rooks = board.pieces(color, Piece::Rook);
            while rooks.any() {
                let sq = rooks.pop_lsb();
                let file_bb = file_mask(sq.file() as usize);
                let own_on_file = (own_pawns & file_bb).any();
                let opp_on_file = (opp_pawns & file_bb).any();
                if !own_on_file && !opp_on_file {
                    mg[c] += 25;
                    eg[c] += 10;
                } else if !own_on_file {
                    mg[c] += 12;
                    eg[c] += 8;
                }
                let seventh = if color == Color::White {
                    6usize
                } else {
                    1usize
                };
                if sq.rank() as usize == seventh {
                    mg[c] += 20;
                    eg[c] += 40;
                }
            }

            let pawn_attacks_opp = pawn_attacks(board, opp);
            let safe = !pawn_attacks_opp;

            let mut knights = board.pieces(color, Piece::Knight);
            while knights.any() {
                let sq = knights.pop_lsb();
                let mob = (atk.knight(sq) & safe).count() as i32;
                mg[c] += 4 * mob;
                eg[c] += 4 * mob;
                let rank = sq.rank() as usize;
                let outpost_rank = if color == Color::White { 4 } else { 3 };
                if rank >= outpost_rank
                    && (own_pawns & atk.pawn(color, sq)).any()
                    && !(opp_pawns & atk.pawn(opp, sq)).any()
                {
                    mg[c] += 25;
                    eg[c] += 15;
                }
            }

            let mut bishops = board.pieces(color, Piece::Bishop);
            while bishops.any() {
                let sq = bishops.pop_lsb();
                let moves = atk.bishop(sq, board.all_occ) & !board.color_occ(color);
                let mob = (moves & safe).count() as i32;
                mg[c] += 5 * mob;
                eg[c] += 5 * mob;

                // Bad bishop: own pawns on same color squares restrict the bishop
                let sq_color = if (Bitboard::LIGHT_SQUARES & Bitboard::from(sq)).any() {
                    Bitboard::LIGHT_SQUARES
                } else {
                    Bitboard::DARK_SQUARES
                };
                let blocked = (own_pawns & sq_color).count() as i32;
                mg[c] -= 2 * blocked;
                eg[c] -= 5 * blocked;

                // Trapped bishop: very low mobility → heavy penalty
                let total_mob = moves.count() as i32;
                if total_mob <= 1 {
                    mg[c] -= 80;
                    eg[c] -= 60;
                } else if total_mob == 2 {
                    mg[c] -= 30;
                    eg[c] -= 20;
                }
            }

            let mut rooks = board.pieces(color, Piece::Rook);
            while rooks.any() {
                let sq = rooks.pop_lsb();
                let mob = (atk.rook(sq, board.all_occ) & safe).count() as i32;
                mg[c] += 2 * mob;
                eg[c] += 4 * mob;
            }

            let mut queens = board.pieces(color, Piece::Queen);
            while queens.any() {
                let sq = queens.pop_lsb();
                let mob = ((atk.bishop(sq, board.all_occ) | atk.rook(sq, board.all_occ)) & safe)
                    .count() as i32;
                mg[c] += mob;
                eg[c] += 2 * mob;
            }

            let pawn_atk_bb = pawn_attacks(board, color);
            for &pt in &[Piece::Knight, Piece::Bishop] {
                let threatened = (pawn_atk_bb & board.pieces(opp, pt)).count() as i32;
                mg[c] += 18 * threatened;
                eg[c] += 12 * threatened;
            }
            let threatened_rooks = (pawn_atk_bb & board.pieces(opp, Piece::Rook)).count() as i32;
            mg[c] += 28 * threatened_rooks;
            eg[c] += 18 * threatened_rooks;
            let threatened_queens = (pawn_atk_bb & board.pieces(opp, Piece::Queen)).count() as i32;
            mg[c] += 45 * threatened_queens;
            eg[c] += 30 * threatened_queens;

            let king_sq = board.king_sq(color);
            let king_zone = atk.king(king_sq) | Bitboard::from(king_sq);
            let mut attack_units = 0i32;
            for &(pt, units) in &[
                (Piece::Knight, 2),
                (Piece::Bishop, 2),
                (Piece::Rook, 3),
                (Piece::Queen, 5),
            ] {
                let mut attackers = board.pieces(opp, pt);
                while attackers.any() {
                    let sq = attackers.pop_lsb();
                    let attacks = match pt {
                        Piece::Knight => atk.knight(sq),
                        Piece::Bishop => atk.bishop(sq, board.all_occ),
                        Piece::Rook => atk.rook(sq, board.all_occ),
                        Piece::Queen => atk.bishop(sq, board.all_occ) | atk.rook(sq, board.all_occ),
                        _ => Bitboard::EMPTY,
                    };
                    if (attacks & king_zone).any() {
                        attack_units += units;
                    }
                }
            }
            mg[c] -= SAFETY_TABLE[attack_units.min(15) as usize];

            let kf = king_sq.file() as usize;
            if kf <= 2 || kf >= 5 {
                let start_file = if kf <= 2 { 0usize } else { 5usize };
                let end_file = start_file + 3;
                let rank2 = if color == Color::White { 1u8 } else { 6u8 };
                let rank3 = if color == Color::White { 2u8 } else { 5u8 };
                let mut shelter = 0i32;
                for file in start_file..end_file {
                    let mask = file_mask(file);
                    if (own_pawns & mask & rank_mask(rank2)).any() {
                        shelter += 4;
                    } else if (own_pawns & mask & rank_mask(rank3)).any() {
                        shelter += 2;
                    }
                }
                mg[c] += shelter;
            }

            // King open-file penalty: penalise open/semi-open files near the king
            {
                let kf = king_sq.file() as usize;
                let start = kf.saturating_sub(1);
                let end = (kf + 2).min(8);
                for file in start..end {
                    let file_pawns = own_pawns & file_mask(file);
                    if file_pawns.is_empty() {
                        // Fully open file near king
                        mg[c] -= 18;
                    }
                }
            }
        }

        let (pmg, peg, passers) = self.pawn_structure(board);
        mg[0] += pmg;
        eg[0] += peg;
        let (rmg, reg) = rook_passer_bonus(board, passers);
        mg[0] += rmg;
        eg[0] += reg;

        // Passed pawn king proximity (endgame only)
        for color in [Color::White, Color::Black] {
            let c = color as usize;
            let opp = !color;
            let own_king = board.king_sq(color);
            let opp_king = board.king_sq(opp);
            let mut pp = passers[color as usize];
            while pp.any() {
                let psq = pp.pop_lsb();
                let rel_rank = if color == Color::White {
                    psq.rank() as i32
                } else {
                    7 - psq.rank() as i32
                };
                let own_dist = king_dist(own_king, psq);
                let opp_dist = king_dist(opp_king, psq);
                eg[c] += (opp_dist - own_dist) * (2 + rel_rank);
            }
        }

        // Space: center squares (files C-F) not occupied by own pawns, not attacked by enemy pawns
        {
            let center_files =
                file_mask(2) | file_mask(3) | file_mask(4) | file_mask(5);
            let white_space_ranks = rank_mask(1) | rank_mask(2) | rank_mask(3);
            let black_space_ranks = rank_mask(4) | rank_mask(5) | rank_mask(6);
            let wp = board.pieces(Color::White, Piece::Pawn);
            let bp = board.pieces(Color::Black, Piece::Pawn);
            let black_pawn_atk = pawn_attacks(board, Color::Black);
            let white_pawn_atk = pawn_attacks(board, Color::White);
            let wspace = (center_files & white_space_ranks) & !wp & !black_pawn_atk;
            let bspace = (center_files & black_space_ranks) & !bp & !white_pawn_atk;
            mg[0] += wspace.count() as i32 * 2;
            mg[1] += bspace.count() as i32 * 2;
        }

        for color in [Color::White, Color::Black] {
            let c = color as usize;
            let opp = !color;
            let opp_pawn_atk = pawn_attacks(board, opp);
            for &(pt, pen_mg, pen_eg) in &[
                (Piece::Knight, 45, 30),
                (Piece::Bishop, 45, 30),
                (Piece::Rook, 60, 45),
                (Piece::Queen, 80, 60),
            ] {
                let hanging = (board.pieces(color, pt) & opp_pawn_atk).count() as i32;
                mg[c] -= pen_mg * hanging;
                eg[c] -= pen_eg * hanging;
            }

            // Fully hanging pieces: attacked by any enemy, not defended by own side
            const HANG_PEN: [i32; 6] = [0, 45, 45, 60, 80, 0];
            let non_pawns = board.color_occ(color)
                & !board.pieces(color, Piece::Pawn)
                & !board.pieces(color, Piece::King);
            let mut pieces = non_pawns;
            while pieces.any() {
                let sq = pieces.pop_lsb();
                let all_att = board.attackers_to(sq, board.all_occ);
                if (all_att & board.color_occ(opp)).is_empty() {
                    continue;
                }
                if (all_att & board.color_occ(color)).any() {
                    continue;
                }
                if let Some(pt) = board.piece_type_at(sq) {
                    mg[c] -= HANG_PEN[pt as usize];
                    eg[c] -= HANG_PEN[pt as usize];
                }
            }
        }

        let score_eg = eg[0] - eg[1];
        if phase <= 6 && score_eg.abs() > 200 {
            let winning_side = if score_eg > 0 {
                Color::White
            } else {
                Color::Black
            };
            let losing_king = board.king_sq(!winning_side);
            let rank = losing_king.rank() as i32;
            let file = losing_king.file() as i32;
            let edge = rank.min(7 - rank) + file.min(7 - file);
            let push = (6 - edge).max(0) * 10;
            mg[winning_side as usize] += push;
            eg[winning_side as usize] += push;
        }

        (mg[0] - mg[1], eg[0] - eg[1], phase)
    }

    fn pawn_structure(&mut self, board: &Board) -> (i32, i32, [Bitboard; 2]) {
        let key = board.pawn_hash();
        let idx = (key as usize) & (PAWN_CACHE_SIZE - 1);
        if self.pawn_cache[idx].key == key {
            let entry = self.pawn_cache[idx];
            return (entry.mg, entry.eg, entry.passers);
        }

        let wp = board.pieces(Color::White, Piece::Pawn);
        let bp = board.pieces(Color::Black, Piece::Pawn);
        let mut mg = 0i32;
        let mut eg = 0i32;
        let mut passers = [Bitboard::EMPTY; 2];

        for color in [Color::White, Color::Black] {
            let sign = if color == Color::White { 1 } else { -1 };
            let ours = if color == Color::White { wp } else { bp };
            let theirs = if color == Color::White { bp } else { wp };
            let enemy_atks = pawn_attacks(board, !color);

            // Doubled pawns: penalise once per *extra* pawn on a file (not per pawn)
            for f in 0..8usize {
                let file_bb = ours & file_mask(f);
                let n = file_bb.count();
                if n > 1 {
                    mg += sign * -(n as i32 - 1) * 10;
                    eg += sign * -(n as i32 - 1) * 20;
                }
            }

            let mut bb = ours;
            while bb.any() {
                let sq = bb.pop_lsb();
                let file = sq.file() as usize;
                let rank = sq.rank() as usize;
                let _file_mask_bb = file_mask(file);
                let adjmask = adj_files_mask(file);

                if (ours & adjmask).is_empty() {
                    mg += sign * -15;
                    eg += sign * -20;
                }
                let same_rank_adj = adjmask & rank_mask(sq.rank() as u8);
                if (ours & same_rank_adj).any() {
                    mg += sign * 7;
                    eg += sign * 5;
                }
                if (theirs & passed_pawn_mask(color, sq)).is_empty() {
                    let rank_idx = if color == Color::White {
                        rank
                    } else {
                        7 - rank
                    };
                    mg += sign * PASSED_MG[rank_idx];
                    eg += sign * PASSED_EG[rank_idx];
                    passers[color as usize] |= Bitboard::from(sq);
                }

                let fwd = if color == Color::White {
                    sq.0 + 8
                } else {
                    sq.0.wrapping_sub(8)
                };
                if fwd < 64 {
                    let fwd_sq = Square(fwd);
                    if (ours & ATTACKS.pawn(color, fwd_sq)).is_empty()
                        && (enemy_atks & Bitboard::from(fwd_sq)).any()
                    {
                        mg += sign * -10;
                        eg += sign * -15;
                    }
                }
            }
        }

        self.pawn_cache[idx] = PawnEntry {
            key,
            mg,
            eg,
            passers,
        };
        (mg, eg, passers)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

fn rook_passer_bonus(board: &Board, passers: [Bitboard; 2]) -> (i32, i32) {
    let mut mg = 0i32;
    let mut eg = 0i32;
    for color in [Color::White, Color::Black] {
        let sign = if color == Color::White { 1 } else { -1 };
        let mut pp = passers[color as usize];
        while pp.any() {
            let sq = pp.pop_lsb();
            let file = file_mask(sq.file() as usize);
            if (board.pieces(color, Piece::Rook) & file).any() {
                mg += sign * 15;
                eg += sign * 25;
            }
            if (board.pieces(!color, Piece::Rook) & file).any() {
                mg -= sign * 10;
                eg -= sign * 20;
            }
        }
    }
    (mg, eg)
}

fn pawn_attacks(board: &Board, color: Color) -> Bitboard {
    let mut pawns = board.pieces(color, Piece::Pawn);
    let mut attacks = Bitboard::EMPTY;
    while pawns.any() {
        attacks |= ATTACKS.pawn(color, pawns.pop_lsb());
    }
    attacks
}

fn file_mask(file: usize) -> Bitboard {
    Bitboard(0x0101_0101_0101_0101u64 << file)
}

fn adj_files_mask(file: usize) -> Bitboard {
    let mut bb = Bitboard::EMPTY;
    if file > 0 {
        bb |= file_mask(file - 1);
    }
    if file < 7 {
        bb |= file_mask(file + 1);
    }
    bb
}

fn rank_mask(rank: u8) -> Bitboard {
    Bitboard(0xFFu64 << (rank * 8))
}

fn passed_pawn_mask(color: Color, sq: Square) -> Bitboard {
    let files = file_mask(sq.file() as usize) | adj_files_mask(sq.file() as usize);
    let rank = sq.rank() as u8;
    let mut ranks = Bitboard::EMPTY;
    if color == Color::White {
        for r in (rank + 1)..8 {
            ranks |= rank_mask(r);
        }
    } else {
        for r in 0..rank {
            ranks |= rank_mask(r);
        }
    }
    files & ranks
}

fn king_dist(a: Square, b: Square) -> i32 {
    let dr = (a.rank() as i32 - b.rank() as i32).abs();
    let df = (a.file() as i32 - b.file() as i32).abs();
    dr.max(df)
}
