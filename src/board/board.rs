/// Board representation.
///
/// Uses 12 bitboards (one per color×piece), two occupancy bitboards, and an
/// incremental Zobrist hash.  Make/unmake are performed in-place with an
/// internal history stack — no full-struct copies needed.
use std::cell::Cell;
use std::fmt;

use super::attacks::ATTACKS;
use super::bitboard::Bitboard;
use super::movegen::generate_legal_moves;
use super::moves::{
    CAPTURE, CASTLE_KINGSIDE, CASTLE_QUEENSIDE, DOUBLE_PUSH, EN_PASSANT, Move, MoveList,
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
    fullmove: u16,
    hash: u64,
}

const NO_PIECE: u8 = 255;

// -----------------------------------------------------------------------
// Board
// -----------------------------------------------------------------------

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameResult {
    WhiteCheckmates,
    BlackCheckmates,
    Stalemate,
    Draw,
}

/// The chess board.  Square A1 = 0, H8 = 63 (rank-major, little-endian).
#[derive(Clone)]
pub struct Board {
    /// `pieces[color * 6 + piece_type]`
    pieces: [Bitboard; 12],
    /// `occupancy[color]`
    occupancy: [Bitboard; 2],
    /// Union of both occupancy bitboards.
    pub all_occ: Bitboard,
    /// Encoded piece on each square, or 255 for empty.
    mailbox: [u8; 64],
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
    in_check: Cell<bool>,
    in_check_dirty: Cell<bool>,
    checkers: Cell<Bitboard>,
    checkers_dirty: Cell<bool>,
    history: Vec<UnmakeInfo>,
}

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
            mailbox: [NO_PIECE; 64],
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            ep_sq: 255,
            halfmove_clock: 0,
            fullmove: 1,
            hash: 0,
            in_check: Cell::new(false),
            in_check_dirty: Cell::new(true),
            checkers: Cell::new(Bitboard::EMPTY),
            checkers_dirty: Cell::new(true),
            history: Vec::with_capacity(128),
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

        board.refresh_in_check_and_checkers();
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
        decode_piece(self.mailbox[sq.index()])
    }

    /// Piece type only at a given square.
    #[inline(always)]
    pub fn piece_type_at(&self, sq: Square) -> Option<Piece> {
        decode_piece_type(self.mailbox[sq.index()])
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

    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    pub fn occupied_count(&self) -> u32 {
        self.all_occ.count()
    }

    #[inline(always)]
    pub fn occupied(&self) -> Bitboard {
        self.all_occ
    }

    #[inline(always)]
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.piece_type_at(sq)
    }

    #[inline(always)]
    pub fn color_on(&self, sq: Square) -> Option<Color> {
        self.piece_at(sq).map(|(color, _)| color)
    }

    #[inline(always)]
    pub fn moving_piece(&self, mv: Move) -> Piece {
        self.piece_type_at(mv.from_sq())
            .expect("move source must contain a piece")
    }

    #[inline(always)]
    pub fn is_quiet_move(&self, mv: Move) -> bool {
        !mv.is_capture() && !mv.is_promo() && !mv.is_castling()
    }

    #[inline(always)]
    pub fn en_passant(&self) -> Option<Square> {
        self.ep_square()
    }

    pub fn parse_move(&self, input: &str) -> Option<Move> {
        let parsed = Move::from_uci(input)?;
        generate_legal_moves(self)
            .into_iter()
            .find(|mv| mv.same_uci_move(parsed))
    }

    pub fn play_uci(&mut self, input: &str) -> bool {
        if let Some(mv) = self.parse_move(input) {
            self.make_move(mv);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn make_move_unchecked(&mut self, mv: Move) {
        self.make_move(mv);
    }

    pub fn generate_legal_moves(&self) -> Vec<Move> {
        generate_legal_moves(self)
    }

    pub fn generate_legal_movelist(&self) -> MoveList {
        super::movegen::generate_legal_movelist(self)
    }

    pub fn generate_legal_captures(&mut self) -> MoveList {
        super::movegen::generate_captures(self)
    }

    pub fn generate_legal_quiets(&self) -> MoveList {
        super::movegen::generate_quiets(self)
    }

    pub fn perft(&mut self, depth: u32) -> u64 {
        super::movegen::perft(self, depth)
    }

    pub fn captured_piece(&self, mv: Move) -> Option<Piece> {
        if mv.is_en_passant() {
            Some(Piece::Pawn)
        } else {
            self.piece_type_at(mv.to_sq())
        }
    }

    #[inline(always)]
    pub fn is_capture(&self, mv: Move) -> bool {
        mv.is_capture()
    }

    #[inline(always)]
    pub fn is_en_passant(&self, mv: Move) -> bool {
        mv.is_en_passant()
    }

    pub fn gives_check(&mut self, mv: Move) -> bool {
        self.make_move(mv);
        let gives_check = self.is_in_check();
        self.unmake_move(mv);
        gives_check
    }

    pub fn can_declare_draw(&self) -> bool {
        self.halfmove_clock >= 100
            || self.has_insufficient_material()
            || self.is_threefold_repetition()
    }

    pub fn can_declare_draw_in_search(&self) -> bool {
        self.halfmove_clock >= 100 || self.has_insufficient_material() || self.is_repetition(2)
    }

    pub fn has_non_pawn_material(&self, color: Color) -> bool {
        (self.pieces(color, Piece::Knight)
            | self.pieces(color, Piece::Bishop)
            | self.pieces(color, Piece::Rook)
            | self.pieces(color, Piece::Queen))
        .any()
    }

    #[inline(always)]
    pub fn pawn_key(&self) -> u64 {
        self.pieces(Color::White, Piece::Pawn).0
            ^ self.pieces(Color::Black, Piece::Pawn).0.rotate_left(32)
    }

    #[inline(always)]
    pub fn attackers_to_color(&self, sq: Square, occ: Bitboard, color: Color) -> Bitboard {
        self.attackers_to(sq, occ) & self.color_occ(color) & occ
    }

    pub fn see(&self, mv: Move) -> i32 {
        let Some(victim) = self.captured_piece(mv) else {
            return if mv.is_promo() {
                piece_value(mv.promo_piece()) - piece_value(Piece::Pawn)
            } else {
                0
            };
        };

        let target = mv.to_sq();
        let mut occ = self.all_occ;
        let mut side = self.side_to_move;
        let mut gains = [0i32; 32];
        let mut depth = 0usize;

        gains[0] = piece_value(victim);
        if mv.is_promo() {
            gains[0] += piece_value(mv.promo_piece()) - piece_value(Piece::Pawn);
        }

        let from = mv.from_sq();
        let mut attacker_piece = self.moving_piece(mv);
        occ ^= Bitboard::from(from);
        if mv.is_en_passant() {
            let cap_sq = if side == Color::White {
                Square(target.0 - 8)
            } else {
                Square(target.0 + 8)
            };
            occ ^= Bitboard::from(cap_sq);
        } else {
            occ ^= Bitboard::from(target);
        }
        occ |= Bitboard::from(target);

        loop {
            side = !side;
            let mut attackers = self.attackers_to_color(target, occ, side);
            if attackers.is_empty() {
                break;
            }

            let (sq, piece) = self.least_valuable_attacker(attackers, side);
            depth += 1;
            gains[depth] = piece_value(attacker_piece) - gains[depth - 1];

            if gains[depth].max(-gains[depth - 1]) < 0 {
                break;
            }

            attacker_piece = piece;
            occ ^= Bitboard::from(sq);
            attackers = self.attackers_to_color(target, occ, !side);
            if (attackers & self.pieces(!side, Piece::King)).any() {
                break;
            }
        }

        while depth > 0 {
            depth -= 1;
            gains[depth] = -gains[depth + 1].max(-gains[depth]);
        }
        gains[0]
    }

    pub fn game_result(&self) -> Option<GameResult> {
        if self.can_declare_draw() {
            return Some(GameResult::Draw);
        }

        if !generate_legal_moves(self).is_empty() {
            return None;
        }

        if self.is_in_check() {
            match self.side_to_move {
                Color::White => Some(GameResult::BlackCheckmates),
                Color::Black => Some(GameResult::WhiteCheckmates),
            }
        } else {
            Some(GameResult::Stalemate)
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
        if self.in_check_dirty.get() {
            self.refresh_check_state();
        }
        self.in_check.get()
    }

    #[inline(always)]
    pub fn checkers(&self) -> Bitboard {
        if self.checkers_dirty.get() {
            self.refresh_check_state();
        }
        self.checkers.get()
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
    #[inline(always)]
    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from_sq();
        let to = mv.to_sq();
        let flags = mv.flags();
        let us = self.side_to_move;
        let them = !us;

        let zob = &ZOBRIST;

        let old_castling = self.castling;
        let old_ep_sq = self.ep_sq;
        let old_halfmove_clock = self.halfmove_clock;
        let old_fullmove = self.fullmove;
        let old_hash = self.hash;
        let mut captured = 255;

        // Halfmove clock: reset on pawn move or capture; increment otherwise.
        // We set it properly below after determining if it's a pawn move.

        // Remove old EP contribution from hash
        if self.ep_sq != 255 {
            self.hash ^= zob.ep(Square(self.ep_sq).file());
        }
        self.ep_sq = 255;

        debug_assert!(self.mailbox[from.index()] < 12);
        let moving_piece = self.piece_type_at_unchecked(from);

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
            captured = encode_piece(them, Piece::Pawn);
            self.remove_piece(them, Piece::Pawn, ep_cap_sq);
            self.hash ^= zob.piece(them, Piece::Pawn, ep_cap_sq);
            self.halfmove_clock = 0;
        } else if flags == CAPTURE || flags >= PROMO_CAPTURE_KNIGHT {
            // Regular capture (including promo-captures)
            debug_assert!(self.mailbox[to.index()] < 12);
            let captured_piece = self.piece_type_at_unchecked(to);
            captured = encode_piece(them, captured_piece);
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
        let new_castling = self.castling.update(from, to);
        if new_castling != self.castling {
            self.hash ^= zob.castling(self.castling) ^ zob.castling(new_castling);
            self.castling = new_castling;
        }

        // Flip side to move
        self.hash ^= zob.side();
        self.side_to_move = them;

        // Fullmove counter
        if us == Color::Black {
            self.fullmove += 1;
        }
        self.history.push(UnmakeInfo {
            captured,
            castling: old_castling,
            ep_sq: old_ep_sq,
            halfmove_clock: old_halfmove_clock,
            fullmove: old_fullmove,
            hash: old_hash,
        });
        self.mark_check_state_dirty();
    }

    pub fn make_null_move(&mut self) {
        debug_assert!(!self.is_in_check(), "null move while in check");
        let old_castling = self.castling;
        let old_ep_sq = self.ep_sq;
        let old_halfmove_clock = self.halfmove_clock;
        let old_fullmove = self.fullmove;
        let old_hash = self.hash;

        if self.ep_sq != 255 {
            self.hash ^= ZOBRIST.ep(Square(self.ep_sq).file());
            self.ep_sq = 255;
        }
        if self.side_to_move == Color::Black {
            self.fullmove += 1;
        }
        self.halfmove_clock = self.halfmove_clock.saturating_add(1);
        self.side_to_move = !self.side_to_move;
        self.hash ^= ZOBRIST.side();
        self.history.push(UnmakeInfo {
            captured: NO_PIECE,
            castling: old_castling,
            ep_sq: old_ep_sq,
            halfmove_clock: old_halfmove_clock,
            fullmove: old_fullmove,
            hash: old_hash,
        });
        self.mark_check_state_dirty();
    }

    pub fn unmake_null_move(&mut self) {
        let info = self
            .history
            .pop()
            .expect("unmake_null_move with empty history");
        debug_assert_eq!(info.captured, NO_PIECE);
        self.side_to_move = !self.side_to_move;
        self.castling = info.castling;
        self.ep_sq = info.ep_sq;
        self.halfmove_clock = info.halfmove_clock;
        self.fullmove = info.fullmove;
        self.hash = info.hash;
        self.mark_check_state_dirty();
    }

    /// Undo the last move.
    #[inline(always)]
    pub fn unmake_move(&mut self, mv: Move) {
        let info = self.history.pop().expect("unmake_move with empty history");

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
        self.fullmove = info.fullmove;
        self.hash = info.hash;
        self.mark_check_state_dirty();

        // Move the piece back from `to` to `from`
        let moved_piece = if flags >= PROMO_KNIGHT {
            // Promotion: remove the promo piece, restore a pawn
            let promo = mv.promo_piece();
            self.remove_piece(us, promo, to);
            Piece::Pawn
        } else {
            debug_assert!(self.mailbox[to.index()] < 12);
            let p = self.piece_type_at_unchecked(to);
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
    // Internal helpers
    // -----------------------------------------------------------------------

    #[inline(always)]
    fn add_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let bb = Bitboard::from(sq);
        self.mailbox[sq.index()] = encode_piece(color, piece);
        self.pieces[color as usize * 6 + piece as usize] |= bb;
        self.occupancy[color as usize] |= bb;
        self.all_occ |= bb;
    }

    #[inline(always)]
    fn remove_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let bb = Bitboard::from(sq);
        self.mailbox[sq.index()] = NO_PIECE;
        self.pieces[color as usize * 6 + piece as usize] ^= bb;
        self.occupancy[color as usize] ^= bb;
        self.all_occ ^= bb;
    }

    #[inline(always)]
    fn piece_type_at_unchecked(&self, sq: Square) -> Piece {
        Piece::ALL[(self.mailbox[sq.index()] % 6) as usize]
    }

    fn least_valuable_attacker(&self, attackers: Bitboard, color: Color) -> (Square, Piece) {
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let bb = attackers & self.pieces(color, piece);
            if bb.any() {
                return (bb.lsb(), piece);
            }
        }
        unreachable!("least_valuable_attacker called with no attackers")
    }

    #[inline(always)]
    fn calculate_checkers(&self) -> Bitboard {
        self.attackers_to(self.king_sq(self.side_to_move), self.all_occ)
            & self.color_occ(!self.side_to_move)
    }

    #[inline(always)]
    fn refresh_in_check_and_checkers(&mut self) {
        let checkers = self.calculate_checkers();
        self.in_check.set(checkers.any());
        self.in_check_dirty.set(false);
        self.checkers.set(checkers);
        self.checkers_dirty.set(false);
    }

    #[inline(always)]
    fn refresh_check_state(&self) {
        let checkers = self.calculate_checkers();
        self.in_check.set(checkers.any());
        self.in_check_dirty.set(false);
        self.checkers.set(checkers);
        self.checkers_dirty.set(false);
    }

    #[inline(always)]
    fn mark_check_state_dirty(&self) {
        self.in_check_dirty.set(true);
        self.checkers_dirty.set(true);
    }

    fn has_insufficient_material(&self) -> bool {
        if (self.pieces(Color::White, Piece::Pawn)
            | self.pieces(Color::Black, Piece::Pawn)
            | self.pieces(Color::White, Piece::Rook)
            | self.pieces(Color::Black, Piece::Rook)
            | self.pieces(Color::White, Piece::Queen)
            | self.pieces(Color::Black, Piece::Queen))
        .any()
        {
            return false;
        }

        let knights =
            self.pieces(Color::White, Piece::Knight) | self.pieces(Color::Black, Piece::Knight);
        let bishops =
            self.pieces(Color::White, Piece::Bishop) | self.pieces(Color::Black, Piece::Bishop);
        let minors = knights | bishops;
        if minors.count() <= 1 {
            return true;
        }
        if knights.any() {
            return false;
        }

        let mut bishop_squares = bishops;
        let mut color_complex: Option<u8> = None;
        while bishop_squares.any() {
            let sq = bishop_squares.pop_lsb();
            let complex = (sq.file() as u8 + sq.rank() as u8) & 1;
            if color_complex.is_some_and(|known| known != complex) {
                return false;
            }
            color_complex = Some(complex);
        }
        true
    }

    fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    fn is_repetition(&self, needed_count: usize) -> bool {
        1 + self
            .history
            .iter()
            .rev()
            .take(self.halfmove_clock as usize)
            .filter(|position| position.hash == self.hash)
            .count()
            >= needed_count
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::starting_position()
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

#[inline(always)]
fn encode_piece(color: Color, piece: Piece) -> u8 {
    color as u8 * 6 + piece as u8
}

#[inline(always)]
fn decode_piece(encoded: u8) -> Option<(Color, Piece)> {
    if encoded >= 12 {
        return None;
    }

    let color = if encoded < 6 {
        Color::White
    } else {
        Color::Black
    };
    let piece = Piece::ALL[(encoded % 6) as usize];
    Some((color, piece))
}

#[inline(always)]
fn decode_piece_type(encoded: u8) -> Option<Piece> {
    if encoded < 12 {
        Some(Piece::ALL[(encoded % 6) as usize])
    } else {
        None
    }
}

#[inline(always)]
fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000,
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
