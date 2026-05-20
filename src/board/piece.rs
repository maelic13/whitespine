use std::fmt;
use std::ops::Not;

use super::square::{Rank, Square};

/// Side to move.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Not for Color {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Color {
    /// Relative rank for this color (rank 1 for white = rank 8 for black).
    #[inline(always)]
    pub fn relative_rank(self, rank: Rank) -> Rank {
        match self {
            Self::White => rank,
            Self::Black => Rank::from_u8(7 - rank as u8),
        }
    }

    /// Relative square (flips rank for black).
    #[inline(always)]
    pub fn relative_square(self, sq: Square) -> Square {
        match self {
            Self::White => sq,
            Self::Black => sq.flip_rank(),
        }
    }
}

// -----------------------------------------------------------------------
// Piece type
// -----------------------------------------------------------------------

/// Six piece types (no color information).
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    pub const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];

    /// Convert a promotion char ('n', 'b', 'r', 'q') to a piece.
    pub fn from_promo_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'n' => Some(Self::Knight),
            'b' => Some(Self::Bishop),
            'r' => Some(Self::Rook),
            'q' => Some(Self::Queen),
            _ => None,
        }
    }

    pub fn promo_char(self) -> char {
        match self {
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            _ => '?',
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        };
        write!(f, "{c}")
    }
}

// -----------------------------------------------------------------------
// Castling rights
// -----------------------------------------------------------------------

/// 4-bit flags: bit 0 = white kingside, bit 1 = white queenside,
///              bit 2 = black kingside, bit 3 = black queenside.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Hash)]
pub struct CastlingRights(pub u8);

impl CastlingRights {
    pub const NONE: Self = Self(0);
    pub const WHITE_KINGSIDE: Self = Self(1);
    pub const WHITE_QUEENSIDE: Self = Self(2);
    pub const BLACK_KINGSIDE: Self = Self(4);
    pub const BLACK_QUEENSIDE: Self = Self(8);
    pub const ALL: Self = Self(15);
    pub const WHITE_ALL: Self = Self(3);
    pub const BLACK_ALL: Self = Self(12);

    /// Per-square castling update masks.
    /// When a piece moves from/to a square, AND the rights with this mask.
    pub const UPDATE_MASK: [u8; 64] = {
        let mut m = [0xF_u8; 64];
        // White rooks / king
        m[0] = 0xF & !2; // A1 = white queenside rook → clear WHITE_QUEENSIDE
        m[4] = 0xF & !3; // E1 = white king            → clear WK + WQ
        m[7] = 0xF & !1; // H1 = white kingside rook  → clear WHITE_KINGSIDE
        // Black rooks / king
        m[56] = 0xF & !8; // A8 = black queenside rook → clear BLACK_QUEENSIDE
        m[60] = 0xF & !12; // E8 = black king          → clear BK + BQ
        m[63] = 0xF & !4; // H8 = black kingside rook  → clear BLACK_KINGSIDE
        m
    };

    #[inline(always)]
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    /// Update rights when a piece is moved from `from` to `to`.
    #[inline(always)]
    pub fn update(self, from: Square, to: Square) -> Self {
        Self(self.0 & Self::UPDATE_MASK[from.index()] & Self::UPDATE_MASK[to.index()])
    }

    pub fn as_str(self) -> &'static str {
        match self.0 {
            0 => "-",
            1 => "K",
            2 => "Q",
            3 => "KQ",
            4 => "k",
            5 => "Kk",
            6 => "Qk",
            7 => "KQk",
            8 => "q",
            9 => "Kq",
            10 => "Qq",
            11 => "KQq",
            12 => "kq",
            13 => "Kkq",
            14 => "Qkq",
            15 => "KQkq",
            _ => unreachable!(),
        }
    }
}
