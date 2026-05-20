/// Board representation.
///
/// Uses 12 bitboards (one per color×piece), two occupancy bitboards, and an
/// incremental Zobrist hash.  Make/unmake are performed in-place with an
/// internal history stack — no full-struct copies needed.
use std::fmt;

use super::attacks::ATTACKS;
use super::bitboard::Bitboard;
use super::moves::{
    CAPTURE, CASTLE_KINGSIDE, CASTLE_QUEENSIDE, DOUBLE_PUSH, EN_PASSANT, Move,
    PROMO_CAPTURE_KNIGHT, PROMO_KNIGHT,
};
use super::piece::{CastlingRights, Color, Piece};
use super::square::Square;
use super::zobrist::ZOBRIST;

// -----------------------------------------------------------------------
// Unmake info — everything needed to undo a move
// -----------------------------------------------------------------------

#[derive(Copy, Clone)]
struct UnmakeInfo {
    /// Captured piece, if any.  255 = no capture.
    captured: u8, // piece index: color*6 + piece, or 255
    castling: CastlingRights,
    ep_sq: u8, // 255 = no EP
    halfmove_clock: u8,
    hash: u64,
}

// -----------------------------------------------------------------------
// Board
// -----------------------------------------------------------------------

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The chess board.  Square A1 = 0, H8 = 63 (rank-major, little-endian).
#[derive(Clone)]
pub struct Board {
    /// `pieces[color * 6 + piece_type]`
    pieces: [Bitboard; 12],
    /// `occupancy[color]`
    occupancy: [Bitboard; 2],
    /// Union of both occupancy bitboards.
    pub all_occ: Bitboard,
    /// Side to move.
    pub side_to_move: Color,
    pub castling: CastlingRights,
    /// En passant target square (the square a capturing pawn moves *to*).
    /// `255` encodes "no EP".
    ep_sq: u8,
    pub halfmove_clock: u8,
    pub fullmove: u16,
    /// Incrementally updated Zobrist hash.
    pub hash: u64,
    /// Incremental material balance in centipawns (positive = White ahead).
    /// Piece values: P=100, N=320, B=330, R=500, Q=900, K=0.
    pub material: i32,
    /// Square-indexed piece table for O(1) lookup.  `255` = empty;
    /// otherwise `color * 6 + piece_type`.
    mailbox: [u8; 64],
    /// Current depth in the history stack.
    ply: usize,
    /// Per-ply undo information.  256 entries covers any realistic search depth.
    history: Box<[UnmakeInfo; 1024]>,
}

/// Centipawn material values indexed by `Piece as usize`.
const PIECE_VALUE: [i32; 6] = [100, 320, 330, 500, 900, 0];

const EMPTY_UNMAKE: UnmakeInfo = UnmakeInfo {
    captured: 255,
    castling: CastlingRights::NONE,
    ep_sq: 255,
    halfmove_clock: 0,
    hash: 0,
};

impl Board {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    pub fn starting_position() -> Self {
        Self::from_fen(STARTING_FEN).expect("starting FEN is valid")
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Self {
            pieces: [Bitboard::EMPTY; 12],
            occupancy: [Bitboard::EMPTY; 2],
            all_occ: Bitboard::EMPTY,
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            ep_sq: 255,
            halfmove_clock: 0,
            fullmove: 1,
            hash: 0,
            material: 0,
            mailbox: [255u8; 64],
            ply: 0,
            history: Box::new([EMPTY_UNMAKE; 1024]),
        };

        let mut parts = fen.split_whitespace();

        // 1. Piece placement
        let placement = parts.next().ok_or("missing piece placement")?;
        let mut sq = 56u8; // start at A8
        for ch in placement.chars() {
            match ch {
                '/' => {
                    if sq % 8 != 0 {
                        return Err(format!("unexpected '/' at square {sq}"));
                    }
                    sq = sq.wrapping_sub(16); // go down a rank
                }
                '1'..='8' => sq += ch as u8 - b'0',
                c => {
                    let (color, piece) = fen_char_to_piece(c)?;
                    board.add_piece(color, piece, Square(sq));
                    board.hash ^= ZOBRIST.piece(color, piece, Square(sq));
                    sq += 1;
                }
            }
        }

        // 2. Side to move
        match parts.next().ok_or("missing side to move")? {
            "w" => board.side_to_move = Color::White,
            "b" => {
                board.side_to_move = Color::Black;
                board.hash ^= ZOBRIST.side();
            }
            s => return Err(format!("invalid side to move: {s}")),
        }

        // 3. Castling rights
        let castling_str = parts.next().ok_or("missing castling rights")?;
        let mut cr = CastlingRights::NONE;
        for c in castling_str.chars() {
            match c {
                'K' => cr.0 |= CastlingRights::WHITE_KINGSIDE.0,
                'Q' => cr.0 |= CastlingRights::WHITE_QUEENSIDE.0,
                'k' => cr.0 |= CastlingRights::BLACK_KINGSIDE.0,
                'q' => cr.0 |= CastlingRights::BLACK_QUEENSIDE.0,
                '-' => {}
                c => return Err(format!("invalid castling char: {c}")),
            }
        }
        board.castling = cr;
        board.hash ^= ZOBRIST.castling(cr);

        // 4. En passant
        let ep_str = parts.next().ok_or("missing ep field")?;
        if ep_str != "-" {
            let sq = Square::from_algebraic(ep_str)
                .ok_or_else(|| format!("invalid ep square: {ep_str}"))?;
            board.ep_sq = sq.0;
            board.hash ^= ZOBRIST.ep(sq.file());
        }

        // 5. Halfmove clock
        if let Some(s) = parts.next() {
            board.halfmove_clock = s.parse::<u8>().unwrap_or(0);
        }

        // 6. Fullmove number
        if let Some(s) = parts.next() {
            board.fullmove = s.parse::<u16>().unwrap_or(1);
        }

        Ok(board)
    }

    /// Serialize the board to a FEN string.
    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(80);

        // Piece placement (rank 8 down to rank 1)
        for rank in (0..8).rev() {
            let mut empty = 0u8;
            for file in 0..8u8 {
                let sq = Square(rank * 8 + file);
                if let Some((color, piece)) = self.piece_at(sq) {
                    if empty > 0 {
                        fen.push((b'0' + empty) as char);
                        empty = 0;
                    }
                    let c = match piece {
                        Piece::Pawn => 'p',
                        Piece::Knight => 'n',
                        Piece::Bishop => 'b',
                        Piece::Rook => 'r',
                        Piece::Queen => 'q',
                        Piece::King => 'k',
                    };
                    fen.push(if color == Color::White {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    });
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push((b'0' + empty) as char);
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(if self.side_to_move == Color::White {
            'w'
        } else {
            'b'
        });
        fen.push(' ');
        fen.push_str(self.castling.as_str());
        fen.push(' ');
        if self.ep_sq == 255 {
            fen.push('-');
        } else {
            fen.push_str(&Square(self.ep_sq).to_string());
        }
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove.to_string());
        fen
    }

    // -----------------------------------------------------------------------
    // Piece accessors
    // -----------------------------------------------------------------------

    /// Bitboard for a specific color + piece type.
    #[inline(always)]
    pub fn pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.pieces[color as usize * 6 + piece as usize]
    }

    /// Bitboard for all pieces of a given color.
    #[inline(always)]
    pub fn color_occ(&self, color: Color) -> Bitboard {
        self.occupancy[color as usize]
    }

    /// Piece type and color at a given square, or `None` if empty.
    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Option<(Color, Piece)> {
        let v = self.mailbox[sq.index()];
        if v == 255 {
            None
        } else {
            let color = if v < 6 { Color::White } else { Color::Black };
            Some((color, Piece::ALL[(v % 6) as usize]))
        }
    }

    /// Piece type only at a given square.
    #[inline(always)]
    pub fn piece_type_at(&self, sq: Square) -> Option<Piece> {
        let v = self.mailbox[sq.index()];
        if v == 255 {
            None
        } else {
            Some(Piece::ALL[(v % 6) as usize])
        }
    }

    /// King square for a given color.
    #[inline(always)]
    pub fn king_sq(&self, color: Color) -> Square {
        self.pieces(color, Piece::King).lsb()
    }

    /// En passant target square, if any.
    #[inline(always)]
    pub fn ep_square(&self) -> Option<Square> {
        if self.ep_sq == 255 {
            None
        } else {
            Some(Square(self.ep_sq))
        }
    }

    // -----------------------------------------------------------------------
    // Check / attack queries
    // -----------------------------------------------------------------------

    /// Is the given square attacked by any piece of `attacker_color`?
    #[inline(always)]
    pub fn is_attacked(&self, sq: Square, attacker: Color) -> bool {
        let occ = self.all_occ;
        let atk = &*ATTACKS;

        // Pawn attacks
        if (atk.pawn(!attacker, sq) & self.pieces(attacker, Piece::Pawn)).any() {
            return true;
        }
        // Knight
        if (atk.knight(sq) & self.pieces(attacker, Piece::Knight)).any() {
            return true;
        }
        // King
        if (atk.king(sq) & self.pieces(attacker, Piece::King)).any() {
            return true;
        }
        // Bishop / Queen (diagonal)
        if (atk.bishop(sq, occ)
            & (self.pieces(attacker, Piece::Bishop) | self.pieces(attacker, Piece::Queen)))
        .any()
        {
            return true;
        }
        // Rook / Queen (orthogonal)
        if (atk.rook(sq, occ)
            & (self.pieces(attacker, Piece::Rook) | self.pieces(attacker, Piece::Queen)))
        .any()
        {
            return true;
        }
        false
    }

    /// Is the side-to-move's king currently in check?
    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        let king_sq = self.king_sq(self.side_to_move);
        self.is_attacked(king_sq, !self.side_to_move)
    }

    /// Bitboard of all pieces that attack the given square (any color).
    pub fn attackers_to(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let atk = &*ATTACKS;
        atk.pawn(Color::Black, sq) & self.pieces(Color::White, Piece::Pawn)
            | atk.pawn(Color::White, sq) & self.pieces(Color::Black, Piece::Pawn)
            | atk.knight(sq)
                & (self.pieces(Color::White, Piece::Knight)
                    | self.pieces(Color::Black, Piece::Knight))
            | atk.king(sq)
                & (self.pieces(Color::White, Piece::King) | self.pieces(Color::Black, Piece::King))
            | atk.bishop(sq, occ)
                & (self.pieces(Color::White, Piece::Bishop)
                    | self.pieces(Color::Black, Piece::Bishop)
                    | self.pieces(Color::White, Piece::Queen)
                    | self.pieces(Color::Black, Piece::Queen))
            | atk.rook(sq, occ)
                & (self.pieces(Color::White, Piece::Rook)
                    | self.pieces(Color::Black, Piece::Rook)
                    | self.pieces(Color::White, Piece::Queen)
                    | self.pieces(Color::Black, Piece::Queen))
    }

    // -----------------------------------------------------------------------
    // Make / Unmake
    // -----------------------------------------------------------------------

    /// Apply a move in-place.  The move must be legal.
    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from_sq();
        let to = mv.to_sq();
        let flags = mv.flags();
        let us = self.side_to_move;
        let them = !us;

        let zob = &*ZOBRIST;

        // Save undo information
        debug_assert!(self.ply < 1024, "search ply overflow");
        self.history[self.ply] = UnmakeInfo {
            captured: 255,
            castling: self.castling,
            ep_sq: self.ep_sq,
            halfmove_clock: self.halfmove_clock,
            hash: self.hash,
        };

        // Halfmove clock: reset on pawn move or capture; increment otherwise.
        // We set it properly below after determining if it's a pawn move.

        // Remove old EP contribution from hash
        if self.ep_sq != 255 {
            self.hash ^= zob.ep(Square(self.ep_sq).file());
        }
        self.ep_sq = 255;

        // Remove old castling from hash
        self.hash ^= zob.castling(self.castling);

        let moving_piece = self
            .piece_type_at(from)
            .expect("make_move: no piece on from square");

        // Remove moving piece from origin
        self.remove_piece(us, moving_piece, from);
        self.hash ^= zob.piece(us, moving_piece, from);

        // Handle en passant capture
        if flags == EN_PASSANT {
            let ep_cap_sq = if us == Color::White {
                Square(to.0 - 8)
            } else {
                Square(to.0 + 8)
            };
            self.history[self.ply].captured = them as u8 * 6 + Piece::Pawn as u8;
            self.remove_piece(them, Piece::Pawn, ep_cap_sq);
            self.hash ^= zob.piece(them, Piece::Pawn, ep_cap_sq);
            self.halfmove_clock = 0;
        } else if flags == CAPTURE || flags >= PROMO_CAPTURE_KNIGHT {
            // Regular capture (including promo-captures)
            let captured_piece = self
                .piece_type_at(to)
                .expect("make_move: no piece on capture square");
            self.history[self.ply].captured = them as u8 * 6 + captured_piece as u8;
            self.remove_piece(them, captured_piece, to);
            self.hash ^= zob.piece(them, captured_piece, to);
            self.halfmove_clock = 0;
        } else if moving_piece == Piece::Pawn {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Place moving piece on destination (or promotion piece)
        if flags >= PROMO_KNIGHT {
            let promo = mv.promo_piece();
            self.add_piece(us, promo, to);
            self.hash ^= zob.piece(us, promo, to);
        } else {
            self.add_piece(us, moving_piece, to);
            self.hash ^= zob.piece(us, moving_piece, to);
        }

        // Castling: move the rook as well
        match flags {
            CASTLE_KINGSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::H1, Square::F1)
                } else {
                    (Square::H8, Square::F8)
                };
                self.remove_piece(us, Piece::Rook, rook_from);
                self.hash ^= zob.piece(us, Piece::Rook, rook_from);
                self.add_piece(us, Piece::Rook, rook_to);
                self.hash ^= zob.piece(us, Piece::Rook, rook_to);
            }
            CASTLE_QUEENSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::A1, Square::D1)
                } else {
                    (Square::A8, Square::D8)
                };
                self.remove_piece(us, Piece::Rook, rook_from);
                self.hash ^= zob.piece(us, Piece::Rook, rook_from);
                self.add_piece(us, Piece::Rook, rook_to);
                self.hash ^= zob.piece(us, Piece::Rook, rook_to);
            }
            DOUBLE_PUSH => {
                // Set en passant square (one step behind the destination)
                let ep = if us == Color::White {
                    Square(to.0 - 8)
                } else {
                    Square(to.0 + 8)
                };
                self.ep_sq = ep.0;
                self.hash ^= zob.ep(ep.file());
            }
            _ => {}
        }

        // Update castling rights
        self.castling = self.castling.update(from, to);
        self.hash ^= zob.castling(self.castling);

        // Flip side to move
        self.hash ^= zob.side();
        self.side_to_move = them;

        // Fullmove counter
        if us == Color::Black {
            self.fullmove += 1;
        }

        self.ply += 1;
    }

    /// Undo the last move.
    pub fn unmake_move(&mut self, mv: Move) {
        self.ply -= 1;
        let info = self.history[self.ply];

        let from = mv.from_sq();
        let to = mv.to_sq();
        let flags = mv.flags();

        // Restore side to move (it was flipped by make_move)
        self.side_to_move = !self.side_to_move;
        let us = self.side_to_move;
        let _them = !us;

        // Restore state fields
        self.castling = info.castling;
        self.ep_sq = info.ep_sq;
        self.halfmove_clock = info.halfmove_clock;
        self.hash = info.hash;
        if us == Color::Black && self.fullmove > 1 {
            self.fullmove -= 1;
        }

        // Move the piece back from `to` to `from`
        let moved_piece = if flags >= PROMO_KNIGHT {
            // Promotion: remove the promo piece, restore a pawn
            let promo = mv.promo_piece();
            self.remove_piece(us, promo, to);
            Piece::Pawn
        } else {
            let p = self
                .piece_type_at(to)
                .expect("unmake_move: no piece on to square");
            self.remove_piece(us, p, to);
            p
        };

        self.add_piece(us, moved_piece, from);

        // Restore captured piece
        if info.captured != 255 {
            let cap_color = if info.captured < 6 {
                Color::White
            } else {
                Color::Black
            };
            let cap_piece = Piece::ALL[(info.captured % 6) as usize];
            let cap_sq = if flags == EN_PASSANT {
                if us == Color::White {
                    Square(to.0 - 8)
                } else {
                    Square(to.0 + 8)
                }
            } else {
                to
            };
            self.add_piece(cap_color, cap_piece, cap_sq);
        }

        // Undo castling rook move
        match flags {
            CASTLE_KINGSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::H1, Square::F1)
                } else {
                    (Square::H8, Square::F8)
                };
                self.remove_piece(us, Piece::Rook, rook_to);
                self.add_piece(us, Piece::Rook, rook_from);
            }
            CASTLE_QUEENSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::A1, Square::D1)
                } else {
                    (Square::A8, Square::D8)
                };
                self.remove_piece(us, Piece::Rook, rook_to);
                self.add_piece(us, Piece::Rook, rook_from);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Null move (for Null Move Pruning)
    // -----------------------------------------------------------------------

    /// Apply a null move — pass the turn without moving a piece.
    ///
    /// Used in Null Move Pruning (NMP).  Only legal when the side to move is
    /// not in check.  Call [`unmake_null_move`] to undo.
    pub fn make_null_move(&mut self) {
        let zob = &*ZOBRIST;

        self.history[self.ply] = UnmakeInfo {
            captured: 255,
            castling: self.castling,
            ep_sq: self.ep_sq,
            halfmove_clock: self.halfmove_clock,
            hash: self.hash,
        };
        self.ply += 1;

        // Clear EP square
        if self.ep_sq != 255 {
            self.hash ^= zob.ep(Square(self.ep_sq).file());
            self.ep_sq = 255;
        }

        self.hash ^= zob.side();
        self.side_to_move = !self.side_to_move;
        self.halfmove_clock += 1;
    }

    /// Undo a null move made with [`make_null_move`].
    pub fn unmake_null_move(&mut self) {
        self.ply -= 1;
        let info = self.history[self.ply];

        self.side_to_move = !self.side_to_move;
        self.ep_sq = info.ep_sq;
        self.halfmove_clock = info.halfmove_clock;
        self.hash = info.hash;
        // castling rights unchanged by a null move; no pieces moved so no
        // bitboard / mailbox / material updates needed.
    }

    // -----------------------------------------------------------------------
    // Draw detection
    // -----------------------------------------------------------------------

    /// Returns `true` if the current position has appeared at least once
    /// earlier in the search path (2-fold repetition).
    ///
    /// Uses the Zobrist hash stored in the history stack.
    /// Only looks back as far as the last irreversible move (`halfmove_clock`).
    pub fn is_repetition(&self) -> bool {
        let max_back = self.halfmove_clock as usize;
        if max_back < 2 || self.ply < 2 {
            return false;
        }
        // Step back two plies at a time (same side to move).
        let min_k = self.ply.saturating_sub(max_back);
        let mut k = self.ply - 2;
        loop {
            if self.history[k].hash == self.hash {
                return true;
            }
            if k < 2 || k <= min_k {
                break;
            }
            k -= 2;
        }
        false
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    pub fn is_draw(&self) -> bool {
        self.halfmove_clock >= 100 || self.is_repetition()
    }

    pub fn has_non_pawn_material(&self, color: Color) -> bool {
        let base = color as usize * 6;
        (self.pieces[base + 1]
            | self.pieces[base + 2]
            | self.pieces[base + 3]
            | self.pieces[base + 4])
            .any()
    }

    pub fn pawn_hash(&self) -> u64 {
        self.pieces[0].0 ^ self.pieces[6].0
    }

    /// Static Exchange Evaluation (SEE).
    /// Returns estimated material gain of making move `mv`.
    /// Positive = winning capture, negative = losing capture.
    pub fn see(&self, mv: Move) -> i32 {
        use super::moves::{EN_PASSANT, PROMO_KNIGHT};
        const SEE_VAL: [i32; 6] = [100, 300, 300, 500, 900, 20000];

        let to = mv.to_sq();
        let from = mv.from_sq();
        let flags = mv.flags();

        // Value of the piece being captured.
        let target = if flags == EN_PASSANT {
            SEE_VAL[Piece::Pawn as usize]
        } else {
            match self.piece_type_at(to) {
                Some(piece) => SEE_VAL[piece as usize],
                None => {
                    if flags >= PROMO_KNIGHT {
                        // Quiet promotion: net gain = promoted piece − pawn.
                        return SEE_VAL[mv.promo_piece() as usize] - SEE_VAL[Piece::Pawn as usize];
                    }
                    return 0;
                }
            }
        };

        let moving_piece = match self.piece_type_at(from) {
            Some(piece) => piece,
            None => return 0,
        };

        // Value of the piece we moved (promoted piece value for promo-captures).
        let mover_val = if flags >= PROMO_KNIGHT {
            SEE_VAL[mv.promo_piece() as usize]
        } else {
            SEE_VAL[moving_piece as usize]
        };

        // Remove the moving piece (and the EP-captured pawn) from occupancy.
        let mut occ = self.all_occ ^ Bitboard::from(from);
        if flags == EN_PASSANT {
            let cap_sq = if self.side_to_move == Color::White {
                Square(to.0 - 8)
            } else {
                Square(to.0 + 8)
            };
            occ ^= Bitboard::from(cap_sq);
        }

        // We already captured `target`.  Can the opponent profitably recapture?
        // No `.max(0)` here — the initial capture is the move being evaluated and
        // is not optional, so the result can go negative for losing captures.
        target - self.see_exchange(to, mover_val, occ, !self.side_to_move)
    }

    /// Recursive helper for SEE.
    ///
    /// Returns how much `side` gains by recapturing `piece_on_to_val` at `to`
    /// within the given occupancy `occ`, with optimal play from both sides.
    /// Each side may decline a losing trade via the `.max(0)`.
    fn see_exchange(&self, to: Square, piece_on_to_val: i32, occ: Bitboard, side: Color) -> i32 {
        const SEE_VAL: [i32; 6] = [100, 300, 300, 500, 900, 20000];
        let atk = &*ATTACKS;

        for &piece in &Piece::ALL {
            // Least-valuable attacker of `to` for `side` in current `occ`.
            // Sliding pieces use the updated `occ` to reveal X-ray attackers.
            let piece_bb = self.pieces(side, piece) & occ;
            let attackers = match piece {
                Piece::Pawn => atk.pawn(!side, to) & piece_bb,
                Piece::Knight => atk.knight(to) & piece_bb,
                Piece::Bishop => atk.bishop(to, occ) & piece_bb,
                Piece::Rook => atk.rook(to, occ) & piece_bb,
                Piece::Queen => (atk.bishop(to, occ) | atk.rook(to, occ)) & piece_bb,
                Piece::King => atk.king(to) & piece_bb,
            };
            if attackers.any() {
                let lva_sq = Square(attackers.0.trailing_zeros() as u8);
                let lva_val = SEE_VAL[piece as usize];
                let new_occ = occ ^ Bitboard::from(lva_sq);
                // Side captures piece_on_to_val; opponent may then recapture lva_val.
                // `.max(0)` lets this side decline when the trade is losing for them.
                let gain = piece_on_to_val - self.see_exchange(to, lva_val, new_occ, !side);
                return gain.max(0);
            }
        }

        0 // no attacker — cannot recapture
    }

    #[inline(always)]
    fn add_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let idx = color as u8 * 6 + piece as u8;
        let bb = Bitboard::from(sq);
        self.pieces[idx as usize] |= bb;
        self.occupancy[color as usize] |= bb;
        self.all_occ |= bb;
        self.mailbox[sq.index()] = idx;
        self.material += PIECE_VALUE[piece as usize] * if color == Color::White { 1 } else { -1 };
    }

    #[inline(always)]
    fn remove_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let bb = Bitboard::from(sq);
        self.pieces[color as usize * 6 + piece as usize] ^= bb;
        self.occupancy[color as usize] ^= bb;
        self.all_occ ^= bb;
        self.mailbox[sq.index()] = 255;
        self.material -= PIECE_VALUE[piece as usize] * if color == Color::White { 1 } else { -1 };
    }
}

// -----------------------------------------------------------------------
// FEN helper
// -----------------------------------------------------------------------

fn fen_char_to_piece(c: char) -> Result<(Color, Piece), String> {
    match c {
        'P' => Ok((Color::White, Piece::Pawn)),
        'N' => Ok((Color::White, Piece::Knight)),
        'B' => Ok((Color::White, Piece::Bishop)),
        'R' => Ok((Color::White, Piece::Rook)),
        'Q' => Ok((Color::White, Piece::Queen)),
        'K' => Ok((Color::White, Piece::King)),
        'p' => Ok((Color::Black, Piece::Pawn)),
        'n' => Ok((Color::Black, Piece::Knight)),
        'b' => Ok((Color::Black, Piece::Bishop)),
        'r' => Ok((Color::Black, Piece::Rook)),
        'q' => Ok((Color::Black, Piece::Queen)),
        'k' => Ok((Color::Black, Piece::King)),
        c => Err(format!("invalid FEN piece char: {c}")),
    }
}

// -----------------------------------------------------------------------
// Display
// -----------------------------------------------------------------------

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  +-----------------+")?;
        for rank in (0..8).rev() {
            write!(f, "{} | ", rank + 1)?;
            for file in 0..8u8 {
                let sq = Square(rank * 8 + file);
                if let Some((color, piece)) = self.piece_at(sq) {
                    let c = match piece {
                        Piece::Pawn => 'p',
                        Piece::Knight => 'n',
                        Piece::Bishop => 'b',
                        Piece::Rook => 'r',
                        Piece::Queen => 'q',
                        Piece::King => 'k',
                    };
                    let c = if color == Color::White {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    };
                    write!(f, "{c} ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "  +-----------------+")?;
        writeln!(f, "    a b c d e f g h")?;
        writeln!(f, "  Side: {:?}", self.side_to_move)?;
        writeln!(f, "  Castling: {}", self.castling.as_str())?;
        if self.ep_sq != 255 {
            writeln!(f, "  EP: {}", Square(self.ep_sq))?;
        }
        writeln!(f, "  Hash: 0x{:016X}", self.hash)?;
        Ok(())
    }
}
