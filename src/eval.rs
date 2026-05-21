use crate::board::{ATTACKS, Bitboard, Board, Color, GameResult, Piece, Square};

pub const MATE_SCORE: i32 = 32_000;
pub const INF_SCORE: i32 = 32_001;
pub const VALUE_NONE: i32 = 32_002;

const PAWN_TABLE_SIZE: usize = 16_384;
const TOTAL_PHASE: i32 = 24;

const MG_VAL: [i32; 6] = [82, 337, 365, 477, 1025, 0];
const EG_VAL: [i32; 6] = [94, 281, 297, 512, 936, 0];
const PHASE_W: [i32; 6] = [0, 1, 1, 2, 4, 0];

const MG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -35, -1, -20, -23, -15, 24, 38, -22, -26, -4, -4, -10, 3, 3, 33, -12,
    -27, -2, -5, 12, 17, 6, 10, -25, -14, 13, 6, 21, 23, 12, 17, -23, -6, 7, 26, 31, 65, 56, 25,
    -20, 98, 134, 61, 95, 68, 126, 34, -11, 0, 0, 0, 0, 0, 0, 0, 0,
];
const EG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -10, -6, 10, 0, 14, 7, -5, -19, -8, -4, 7, 22, 17, 16, 3, -14, 13, 0,
    -13, 1, -1, -16, 3, -6, 32, 24, 13, 5, -2, 4, 17, 17, 56, 35, 41, 22, 26, 51, 56, 20, 134, 108,
    109, 107, 105, 104, 112, 108, 0, 0, 0, 0, 0, 0, 0, 0,
];
const MG_KNIGHT_PST: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];
const EG_KNIGHT_PST: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10, 9,
    -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23, -3, -1,
    15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15, -22, -18, -50,
    -64,
];
const MG_BISHOP_PST: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];
const EG_BISHOP_PST: [i32; 64] = [
    -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6, 0,
    4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7, -15,
    -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
];
const MG_ROOK_PST: [i32; 64] = [
    -19, -13, 1, 17, 16, 7, -37, -26, -44, -16, -20, -9, -1, 11, -6, -71, -45, -25, -16, -17, 3, 0,
    -5, -33, -36, -26, -12, -1, 9, -7, 6, -23, -24, -11, 7, 26, 24, 35, -8, -20, -5, 19, 26, 36,
    17, 45, 61, 16, 27, 32, 58, 62, 80, 67, 26, 44, 32, 42, 32, 51, 63, 9, 31, 43,
];
const EG_ROOK_PST: [i32; 64] = [
    -9, 2, 3, -1, -5, -13, 4, -20, -6, -6, 0, 2, -9, -9, -11, -3, -4, 0, -5, -1, -7, -12, -8, -16,
    3, 5, 8, 4, -5, -6, -8, -11, 4, 3, 13, 1, 2, 1, -1, 2, 7, 7, 7, 5, 4, -3, -5, -3, 11, 13, 13,
    11, -3, 3, 8, 3, 13, 10, 18, 15, 12, 12, 8, 5,
];
const MG_QUEEN_PST: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];
const EG_QUEEN_PST: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19, 9,
    3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17, 10, 5,
    -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
];
const MG_KING_PST: [i32; 64] = [
    -15, 36, 12, -54, 8, -28, 24, 14, 1, 7, -8, -64, -43, -16, 9, 8, -14, -14, -22, -46, -44, -30,
    -15, -27, -49, -1, -27, -39, -46, -44, -33, -51, -17, -20, -12, -27, -30, -25, -14, -36, -9,
    24, 2, -16, -20, 6, 22, -22, 29, -1, -20, -7, -8, -4, -38, -29, -65, 23, 16, -15, -56, -34, 2,
    13,
];
const EG_KING_PST: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45,
    44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23,
    16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

#[derive(Copy, Clone, Default)]
struct PawnEntry {
    key: u64,
    mg: i32,
    eg: i32,
    passed: [Bitboard; 2],
    attacks: [Bitboard; 2],
}

#[derive(Clone)]
pub struct Evaluator {
    pawn_table: Vec<PawnEntry>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            pawn_table: vec![PawnEntry::default(); PAWN_TABLE_SIZE],
        }
    }
}

impl Evaluator {
    pub fn clear_pawn_table(&mut self) {
        self.pawn_table.fill(PawnEntry::default());
    }

    pub fn evaluate_result(&self, result: GameResult, color: Color, ply: usize) -> i32 {
        let mate = MATE_SCORE - ply as i32;
        match (result, color) {
            (GameResult::WhiteCheckmates, Color::White)
            | (GameResult::BlackCheckmates, Color::Black) => mate,
            (GameResult::WhiteCheckmates, Color::Black)
            | (GameResult::BlackCheckmates, Color::White) => -mate,
            (GameResult::Stalemate, _) | (GameResult::Draw, _) => 0,
        }
    }

    pub fn evaluate(&mut self, board: &Board) -> i32 {
        let mut mg = 0;
        let mut eg = 0;
        let mut phase = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            for piece in Piece::ALL {
                let mut bb = board.pieces(color, piece);
                phase += bb.count() as i32 * PHASE_W[piece as usize];
                while bb.any() {
                    let sq = bb.pop_lsb();
                    mg += sign * (MG_VAL[piece as usize] + mg_pst(piece)[pst_square(color, sq)]);
                    eg += sign * (EG_VAL[piece as usize] + eg_pst(piece)[pst_square(color, sq)]);
                }
            }
        }
        phase = phase.min(TOTAL_PHASE);

        let mut passed = [Bitboard::EMPTY; 2];
        let mut pawn_attacks = [Bitboard::EMPTY; 2];
        let (pawn_mg, pawn_eg) = self.eval_pawns(board, &mut passed, &mut pawn_attacks);
        mg += pawn_mg;
        eg += pawn_eg;

        self.eval_piece_activity(board, &mut mg, &mut eg, &passed, &pawn_attacks);

        let tempo = if board.side_to_move() == Color::White {
            10
        } else {
            -10
        };
        mg += tempo;

        let mut score = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
        score = scale_drawish_endgames(board, score);
        if board.side_to_move() == Color::White {
            score
        } else {
            -score
        }
    }

    fn eval_pawns(
        &mut self,
        board: &Board,
        passed: &mut [Bitboard; 2],
        attacks: &mut [Bitboard; 2],
    ) -> (i32, i32) {
        let key = board.pawn_key();
        let slot = key as usize & (PAWN_TABLE_SIZE - 1);
        let cached = self.pawn_table[slot];
        if cached.key == key {
            *passed = cached.passed;
            *attacks = cached.attacks;
            return (cached.mg, cached.eg);
        }

        let mut mg = 0;
        let mut eg = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let us = color;
            let them = !us;
            let mut pawns = board.pieces(us, Piece::Pawn);
            let our_pawns = pawns;
            let their_pawns = board.pieces(them, Piece::Pawn);
            let mut pawn_attack_map = Bitboard::EMPTY;

            while pawns.any() {
                let sq = pawns.pop_lsb();
                pawn_attack_map |= ATTACKS.pawn(us, sq);
            }
            attacks[us as usize] = pawn_attack_map;

            let passed_mg = [0, 5, 10, 20, 35, 60, 100, 0];
            let passed_eg = [0, 10, 17, 35, 62, 100, 170, 0];
            let mut tmp = our_pawns;
            passed[us as usize] = Bitboard::EMPTY;
            while tmp.any() {
                let sq = tmp.pop_lsb();
                let file = sq.file() as usize;
                let rel_rank = relative_rank(us, sq) as usize;
                let adjacent = adjacent_files(file);

                if (passed_pawn_mask(us, sq) & their_pawns).is_empty() {
                    passed[us as usize] |= Bitboard::from(sq);
                    mg += sign * passed_mg[rel_rank];
                    eg += sign * passed_eg[rel_rank];

                    if (ATTACKS.pawn(them, sq) & our_pawns).any() {
                        mg += sign * 8;
                        eg += sign * (6 + rel_rank as i32 * 4);
                    }

                    if let Some(stop) = forward_square(us, sq)
                        && (board.occupied() & Bitboard::from(stop)).is_empty()
                    {
                        mg += sign * (rel_rank as i32 * 2);
                        eg += sign * (rel_rank as i32 * 6);
                        if board
                            .attackers_to_color(stop, board.occupied(), them)
                            .is_empty()
                        {
                            eg += sign * (rel_rank as i32 * 8);
                        }
                    }
                } else if rel_rank >= 3
                    && (ATTACKS.pawn(them, sq) & our_pawns).any()
                    && (their_pawns & adjacent & forward_ranks(us, sq.rank() as usize)).is_empty()
                {
                    mg += sign * 6;
                    eg += sign * 10;
                }

                let file_bb = file_bb(file);
                if (our_pawns & file_bb).more_than_one() {
                    mg -= sign * 10;
                    eg -= sign * 20;
                }
                if (our_pawns & adjacent).is_empty() {
                    mg -= sign * 15;
                    eg -= sign * 20;
                }
                if (ATTACKS.pawn(them, sq) & our_pawns).any() {
                    mg += sign * 7;
                    eg += sign * 5;
                }

                let stop_sq = if us == Color::White {
                    sq.0.checked_add(8)
                } else {
                    sq.0.checked_sub(8)
                };
                if (our_pawns & passed_pawn_mask(them, sq) & adjacent).is_empty()
                    && let Some(stop) = stop_sq.filter(|sq| *sq < 64)
                    && (ATTACKS.pawn(us, Square(stop)) & their_pawns).any()
                {
                    mg -= sign * 10;
                    eg -= sign * 15;
                }
            }
        }

        self.pawn_table[slot] = PawnEntry {
            key,
            mg,
            eg,
            passed: *passed,
            attacks: *attacks,
        };
        (mg, eg)
    }

    fn eval_piece_activity(
        &self,
        board: &Board,
        mg: &mut i32,
        eg: &mut i32,
        passed: &[Bitboard; 2],
        pawn_attacks: &[Bitboard; 2],
    ) {
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let them = !color;

            if board.pieces(color, Piece::Bishop).more_than_one() {
                *mg += sign * 30;
                *eg += sign * 50;
            }

            let mut rooks = board.pieces(color, Piece::Rook);
            while rooks.any() {
                let sq = rooks.pop_lsb();
                let file = sq.file() as usize;
                let own_file_empty = (board.pieces(color, Piece::Pawn) & file_bb(file)).is_empty();
                let their_file_empty = (board.pieces(them, Piece::Pawn) & file_bb(file)).is_empty();
                if own_file_empty && their_file_empty {
                    *mg += sign * 25;
                    *eg += sign * 10;
                } else if own_file_empty {
                    *mg += sign * 12;
                    *eg += sign * 8;
                }
                if relative_rank(color, sq) == 6 {
                    *mg += sign * 20;
                    *eg += sign * 40;
                }
            }

            let mut knights = board.pieces(color, Piece::Knight);
            while knights.any() {
                let sq = knights.pop_lsb();
                if relative_rank(color, sq) >= 4
                    && (ATTACKS.pawn(them, sq) & board.pieces(color, Piece::Pawn)).any()
                    && (ATTACKS.pawn(color, sq) & board.pieces(them, Piece::Pawn)).is_empty()
                {
                    *mg += sign * 25;
                    *eg += sign * 15;
                }
            }

            let safe = !pawn_attacks[them as usize];
            for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                let mut pieces = board.pieces(color, piece);
                while pieces.any() {
                    let sq = pieces.pop_lsb();
                    let attacks = attacks_for(piece, sq, board.occupied());
                    let mobility = (attacks & safe & !board.color_occ(color)).count() as i32;
                    *mg += sign * mobility * mobility_mg(piece);
                    *eg += sign * mobility * mobility_eg(piece);
                }
            }

            let mut threats = pawn_attacks[color as usize] & board.color_occ(them);
            while threats.any() {
                let sq = threats.pop_lsb();
                match board.piece_on(sq) {
                    Some(Piece::Knight | Piece::Bishop) => {
                        *mg += sign * 18;
                        *eg += sign * 12;
                    }
                    Some(Piece::Rook) => {
                        *mg += sign * 28;
                        *eg += sign * 18;
                    }
                    Some(Piece::Queen) => {
                        *mg += sign * 45;
                        *eg += sign * 30;
                    }
                    _ => {}
                }
            }

            self.eval_piece_threats(board, color, sign, mg, eg);

            self.eval_king_safety(board, color, sign, mg);
            self.eval_rooks_behind_passers(board, color, sign, passed, mg, eg);
            self.eval_hanging_pieces(board, color, sign, mg, eg);
        }

        self.eval_space(board, pawn_attacks, mg);

        let approximate = (*mg + *eg) / 2;
        if approximate.abs() > 200 {
            let winning = if approximate > 0 {
                Color::White
            } else {
                Color::Black
            };
            let losing = !winning;
            let sign = color_sign(winning);
            let lksq = board.king_sq(losing);
            let wksq = board.king_sq(winning);
            let file_push = (3 - lksq.file() as i32).max(lksq.file() as i32 - 4);
            let rank_push = (3 - lksq.rank() as i32).max(lksq.rank() as i32 - 4);
            let king_distance = wksq.chebyshev_distance(lksq) as i32;
            *eg += sign * (5 * (file_push + rank_push) + (14 - king_distance) * 4);
        }
    }

    fn eval_piece_threats(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let them = !color;
        let targets = board.color_occ(them) & !board.pieces(them, Piece::King);
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            let mut pieces = board.pieces(color, piece);
            while pieces.any() {
                let sq = pieces.pop_lsb();
                let mut attacked = attacks_for(piece, sq, board.occupied()) & targets;
                while attacked.any() {
                    let target = attacked.pop_lsb();
                    let Some(victim) = board.piece_on(target) else {
                        continue;
                    };
                    let defended = board
                        .attackers_to_color(target, board.occupied(), them)
                        .any();
                    let base = match victim {
                        Piece::Pawn => 4,
                        Piece::Knight | Piece::Bishop => 14,
                        Piece::Rook => 24,
                        Piece::Queen => 38,
                        Piece::King => 0,
                    };
                    let bonus = if defended { base } else { base * 2 };
                    *mg += sign * bonus;
                    *eg += sign * (bonus / 2);
                }
            }
        }
    }

    fn eval_space(&self, board: &Board, pawn_attacks: &[Bitboard; 2], mg: &mut i32) {
        let center_files = file_bb(2) | file_bb(3) | file_bb(4) | file_bb(5);
        let white_space_ranks = Bitboard::RANK_2 | Bitboard::RANK_3 | Bitboard::RANK_4;
        let black_space_ranks = Bitboard::RANK_5 | Bitboard::RANK_6 | Bitboard::RANK_7;
        let white_space = center_files
            & white_space_ranks
            & !board.pieces(Color::White, Piece::Pawn)
            & !pawn_attacks[Color::Black as usize];
        let black_space = center_files
            & black_space_ranks
            & !board.pieces(Color::Black, Piece::Pawn)
            & !pawn_attacks[Color::White as usize];
        *mg += (white_space.count() as i32 - black_space.count() as i32) * 2;
    }

    fn eval_king_safety(&self, board: &Board, color: Color, sign: i32, mg: &mut i32) {
        let them = !color;
        let king = board.king_sq(color);
        let king_bb = Bitboard::from(king);
        let mut zone = ATTACKS.king(king) | king_bb;
        zone |= if color == Color::White {
            ATTACKS.king(king).north()
        } else {
            ATTACKS.king(king).south()
        };

        let mut units = 0;
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            let mut pieces = board.pieces(them, piece);
            while pieces.any() {
                let sq = pieces.pop_lsb();
                if (attacks_for(piece, sq, board.occupied()) & zone).any() {
                    units += match piece {
                        Piece::Knight | Piece::Bishop => 2,
                        Piece::Rook => 3,
                        Piece::Queen => 5,
                        _ => 0,
                    };
                }
            }
        }
        const SAFETY: [i32; 16] = [
            0, 0, 10, 25, 40, 60, 80, 95, 105, 110, 112, 114, 115, 116, 117, 118,
        ];
        *mg -= sign * SAFETY[units.min(15) as usize];

        let king_file = king.file() as i32;
        if king_file <= 2 || king_file >= 5 {
            let king_rank = king.rank() as i32;
            for df in -1..=1 {
                let file = king_file + df;
                if !(0..8).contains(&file) {
                    continue;
                }
                let pawns = board.pieces(color, Piece::Pawn) & file_bb(file as usize);
                let in_front = pawns & forward_ranks(color, king_rank as usize);
                if in_front.is_empty() {
                    *mg -= sign * if df == 0 { 20 } else { 10 };
                } else {
                    let pawn_sq = if color == Color::White {
                        in_front.lsb()
                    } else {
                        in_front.msb()
                    };
                    let distance = if color == Color::White {
                        pawn_sq.rank() as i32 - king_rank
                    } else {
                        king_rank - pawn_sq.rank() as i32
                    };
                    if distance == 1 {
                        *mg += sign * 15;
                    } else if distance == 2 {
                        *mg += sign * 7;
                    }
                }
            }
        }

        let enemy_pawns = board.pieces(them, Piece::Pawn);
        let mut storm_files = Bitboard::EMPTY;
        let king_file = king.file() as i32;
        for df in -1..=1 {
            let file = king_file + df;
            if (0..8).contains(&file) {
                storm_files |= file_bb(file as usize);
            }
        }
        let mut storm = enemy_pawns & storm_files;
        while storm.any() {
            let pawn = storm.pop_lsb();
            let rel = relative_rank(them, pawn) as i32;
            if rel >= 3 {
                *mg -= sign * (rel * if pawn.file() == king.file() { 7 } else { 4 });
            }
        }
    }

    fn eval_rooks_behind_passers(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        passed: &[Bitboard; 2],
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let them = !color;
        let mut rooks = board.pieces(color, Piece::Rook);
        while rooks.any() {
            let rook = rooks.pop_lsb();
            let file = rook.file() as usize;
            let file_passers = passed[color as usize] & file_bb(file);
            if file_passers.any() {
                let passer = if color == Color::White {
                    file_passers.lsb()
                } else {
                    file_passers.msb()
                };
                let behind = if color == Color::White {
                    rook.rank() < passer.rank()
                } else {
                    rook.rank() > passer.rank()
                };
                if behind {
                    *mg += sign * 15;
                    *eg += sign * 25;
                }
            }

            let mut enemy_rooks = board.pieces(them, Piece::Rook) & file_bb(file);
            while enemy_rooks.any() && file_passers.any() {
                let enemy = enemy_rooks.pop_lsb();
                let passer = if color == Color::White {
                    file_passers.lsb()
                } else {
                    file_passers.msb()
                };
                let behind = if color == Color::White {
                    enemy.rank() < passer.rank()
                } else {
                    enemy.rank() > passer.rank()
                };
                if behind {
                    *mg -= sign * 10;
                    *eg -= sign * 20;
                }
            }
        }
    }

    fn eval_hanging_pieces(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let them = !color;
        let mut pieces = board.color_occ(color)
            & !board.pieces(color, Piece::Pawn)
            & !board.pieces(color, Piece::King);
        while pieces.any() {
            let sq = pieces.pop_lsb();
            if board
                .attackers_to_color(sq, board.occupied(), them)
                .is_empty()
                || board.attackers_to_color(sq, board.occupied(), color).any()
            {
                continue;
            }
            let penalty = match board.piece_on(sq) {
                Some(Piece::Knight | Piece::Bishop) => 45,
                Some(Piece::Rook) => 60,
                Some(Piece::Queen) => 80,
                _ => 0,
            };
            *mg -= sign * penalty;
            *eg -= sign * penalty;
        }
    }
}

pub fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => MATE_SCORE,
    }
}

#[inline(always)]
fn color_sign(color: Color) -> i32 {
    if color == Color::White { 1 } else { -1 }
}

#[inline(always)]
fn pst_square(color: Color, sq: Square) -> usize {
    match color {
        Color::White => sq.index(),
        Color::Black => (sq.0 ^ 56) as usize,
    }
}

fn mg_pst(piece: Piece) -> &'static [i32; 64] {
    match piece {
        Piece::Pawn => &MG_PAWN_PST,
        Piece::Knight => &MG_KNIGHT_PST,
        Piece::Bishop => &MG_BISHOP_PST,
        Piece::Rook => &MG_ROOK_PST,
        Piece::Queen => &MG_QUEEN_PST,
        Piece::King => &MG_KING_PST,
    }
}

fn eg_pst(piece: Piece) -> &'static [i32; 64] {
    match piece {
        Piece::Pawn => &EG_PAWN_PST,
        Piece::Knight => &EG_KNIGHT_PST,
        Piece::Bishop => &EG_BISHOP_PST,
        Piece::Rook => &EG_ROOK_PST,
        Piece::Queen => &EG_QUEEN_PST,
        Piece::King => &EG_KING_PST,
    }
}

fn attacks_for(piece: Piece, sq: Square, occ: Bitboard) -> Bitboard {
    match piece {
        Piece::Pawn => Bitboard::EMPTY,
        Piece::Knight => ATTACKS.knight(sq),
        Piece::Bishop => ATTACKS.bishop(sq, occ),
        Piece::Rook => ATTACKS.rook(sq, occ),
        Piece::Queen => ATTACKS.queen(sq, occ),
        Piece::King => ATTACKS.king(sq),
    }
}

#[inline(always)]
fn mobility_mg(piece: Piece) -> i32 {
    match piece {
        Piece::Knight => 4,
        Piece::Bishop => 5,
        Piece::Rook => 2,
        Piece::Queen => 1,
        _ => 0,
    }
}

#[inline(always)]
fn mobility_eg(piece: Piece) -> i32 {
    match piece {
        Piece::Knight => 4,
        Piece::Bishop => 5,
        Piece::Rook => 4,
        Piece::Queen => 2,
        _ => 0,
    }
}

#[inline(always)]
fn relative_rank(color: Color, sq: Square) -> u8 {
    match color {
        Color::White => sq.rank() as u8,
        Color::Black => 7 - sq.rank() as u8,
    }
}

fn file_bb(file: usize) -> Bitboard {
    Bitboard(Bitboard::FILE_A.0 << file)
}

fn adjacent_files(file: usize) -> Bitboard {
    let mut bb = Bitboard::EMPTY;
    if file > 0 {
        bb |= file_bb(file - 1);
    }
    if file < 7 {
        bb |= file_bb(file + 1);
    }
    bb
}

fn forward_ranks(color: Color, rank: usize) -> Bitboard {
    let mut bb = Bitboard::EMPTY;
    match color {
        Color::White => {
            for r in rank + 1..8 {
                bb |= Bitboard(0xFFu64 << (r * 8));
            }
        }
        Color::Black => {
            for r in 0..rank {
                bb |= Bitboard(0xFFu64 << (r * 8));
            }
        }
    }
    bb
}

fn forward_square(color: Color, sq: Square) -> Option<Square> {
    match color {
        Color::White => sq.0.checked_add(8).filter(|to| *to < 64).map(Square),
        Color::Black => sq.0.checked_sub(8).map(Square),
    }
}

fn passed_pawn_mask(color: Color, sq: Square) -> Bitboard {
    let file = sq.file() as i32;
    let rank = sq.rank() as i32;
    let mut bb = Bitboard::EMPTY;
    for df in -1..=1 {
        let f = file + df;
        if !(0..8).contains(&f) {
            continue;
        }
        match color {
            Color::White => {
                for r in rank + 1..8 {
                    bb |= Bitboard::from(Square((r * 8 + f) as u8));
                }
            }
            Color::Black => {
                for r in 0..rank {
                    bb |= Bitboard::from(Square((r * 8 + f) as u8));
                }
            }
        }
    }
    bb
}

fn scale_drawish_endgames(board: &Board, mut score: i32) -> i32 {
    let white_bishops = board.pieces(Color::White, Piece::Bishop);
    let black_bishops = board.pieces(Color::Black, Piece::Bishop);
    if white_bishops.any()
        && !white_bishops.more_than_one()
        && black_bishops.any()
        && !black_bishops.more_than_one()
    {
        let white_dark = (white_bishops & Bitboard::DARK_SQUARES).any();
        let black_dark = (black_bishops & Bitboard::DARK_SQUARES).any();
        if white_dark != black_dark {
            let pawns = (board.pieces(Color::White, Piece::Pawn)
                | board.pieces(Color::Black, Piece::Pawn))
            .count() as i32;
            let scale = 32 + pawns * 4;
            score = score * scale.min(48) / 48;
        }
    }

    if has_only_king(board, Color::White) && has_only_knights(board, Color::Black, 2) {
        return 0;
    }
    if has_only_king(board, Color::Black) && has_only_knights(board, Color::White, 2) {
        return 0;
    }

    score
}

fn has_only_king(board: &Board, color: Color) -> bool {
    board.color_occ(color) == Bitboard::from(board.king_sq(color))
}

fn has_only_knights(board: &Board, color: Color, count: u32) -> bool {
    board.pieces(color, Piece::Pawn).is_empty()
        && board.pieces(color, Piece::Bishop).is_empty()
        && board.pieces(color, Piece::Rook).is_empty()
        && board.pieces(color, Piece::Queen).is_empty()
        && board.pieces(color, Piece::Knight).count() == count
}
