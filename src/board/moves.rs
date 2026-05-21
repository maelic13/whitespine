use std::{fmt, slice};

use super::piece::Piece;
use super::square::Square;

/// Move flags — classic CPW 4-bit encoding.
///
/// bits 0-5:  from square
/// bits 6-11: to square
/// bits 12-15: flag (see constants below)
#[derive(Copy, Clone, PartialEq, Eq, Default, Hash, Debug)]
pub struct Move(pub u16);

#[derive(Clone)]
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}

impl MoveList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len] = mv;
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn iter(&self) -> slice::Iter<'_, Move> {
        self.as_slice().iter()
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }
}

impl Default for MoveList {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = slice::Iter<'a, Move>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// Move flag constants (upper 4 bits of a Move)
pub const QUIET: u16 = 0;
pub const DOUBLE_PUSH: u16 = 1;
pub const CASTLE_KINGSIDE: u16 = 2;
pub const CASTLE_QUEENSIDE: u16 = 3;
pub const CAPTURE: u16 = 4;
pub const EN_PASSANT: u16 = 5;
// 6, 7 unused
pub const PROMO_KNIGHT: u16 = 8;
pub const PROMO_BISHOP: u16 = 9;
pub const PROMO_ROOK: u16 = 10;
pub const PROMO_QUEEN: u16 = 11;
pub const PROMO_CAPTURE_KNIGHT: u16 = 12;
pub const PROMO_CAPTURE_BISHOP: u16 = 13;
pub const PROMO_CAPTURE_ROOK: u16 = 14;
pub const PROMO_CAPTURE_QUEEN: u16 = 15;

impl Move {
    pub const NULL: Self = Self(0);

    #[inline(always)]
    pub fn new(from: Square, to: Square, flags: u16) -> Self {
        Self(from.0 as u16 | ((to.0 as u16) << 6) | (flags << 12))
    }

    #[inline(always)]
    pub fn from_sq(self) -> Square {
        Square((self.0 & 0x3F) as u8)
    }

    #[inline(always)]
    pub fn source(self) -> Square {
        self.from_sq()
    }

    #[inline(always)]
    pub fn to_sq(self) -> Square {
        Square(((self.0 >> 6) & 0x3F) as u8)
    }

    #[inline(always)]
    pub fn dest(self) -> Square {
        self.to_sq()
    }

    #[inline(always)]
    pub fn flags(self) -> u16 {
        self.0 >> 12
    }

    #[inline(always)]
    pub fn is_capture(self) -> bool {
        // flags 4,5,12-15 are captures
        let f = self.flags();
        f == CAPTURE || f == EN_PASSANT || f >= PROMO_CAPTURE_KNIGHT
    }

    #[inline(always)]
    pub fn is_promo(self) -> bool {
        self.flags() >= PROMO_KNIGHT
    }

    #[inline(always)]
    pub fn is_castling(self) -> bool {
        let f = self.flags();
        f == CASTLE_KINGSIDE || f == CASTLE_QUEENSIDE
    }

    #[inline(always)]
    pub fn is_en_passant(self) -> bool {
        self.flags() == EN_PASSANT
    }

    #[inline(always)]
    pub fn is_quiet(self) -> bool {
        self.flags() == QUIET
    }

    #[inline(always)]
    pub fn is_double_push(self) -> bool {
        self.flags() == DOUBLE_PUSH
    }

    /// Promotion piece (only valid when `is_promo()` is true).
    #[inline(always)]
    pub fn promo_piece(self) -> Piece {
        match self.flags() & 3 {
            0 => Piece::Knight,
            1 => Piece::Bishop,
            2 => Piece::Rook,
            _ => Piece::Queen,
        }
    }

    #[inline(always)]
    pub fn promotion(self) -> Option<Piece> {
        if self.is_promo() {
            Some(self.promo_piece())
        } else {
            None
        }
    }

    pub fn from_uci(input: &str) -> Option<Move> {
        if input.len() != 4 && input.len() != 5 {
            return None;
        }
        let from = Square::from_algebraic(&input[0..2])?;
        let to = Square::from_algebraic(&input[2..4])?;
        let flags = if input.len() == 5 {
            match Piece::from_promo_char(input.as_bytes()[4] as char)? {
                Piece::Knight => PROMO_KNIGHT,
                Piece::Bishop => PROMO_BISHOP,
                Piece::Rook => PROMO_ROOK,
                Piece::Queen => PROMO_QUEEN,
                _ => return None,
            }
        } else {
            QUIET
        };
        Some(Move::new(from, to, flags))
    }

    #[inline(always)]
    pub fn same_uci_move(self, other: Move) -> bool {
        self.from_sq() == other.from_sq()
            && self.to_sq() == other.to_sq()
            && self.promotion() == other.promotion()
    }

    /// Is this a null move sentinel?
    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            return write!(f, "0000");
        }
        write!(f, "{}{}", self.from_sq(), self.to_sq())?;
        if self.is_promo() {
            write!(f, "{}", self.promo_piece().promo_char())?;
        }
        Ok(())
    }
}
