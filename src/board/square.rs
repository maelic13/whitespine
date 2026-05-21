use std::fmt;

/// A square on the board. A1 = 0, B1 = 1, …, H8 = 63.
/// Encoding: `index = rank * 8 + file` (rank-major, little-endian ranks).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Square(pub u8);

/// File (column) A–H.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

/// Rank (row) 1–8.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[repr(u8)]
pub enum Rank {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}

// -----------------------------------------------------------------------
// Square constants
// -----------------------------------------------------------------------

impl Square {
    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);

    pub const A2: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const C2: Self = Self(10);
    pub const D2: Self = Self(11);
    pub const E2: Self = Self(12);
    pub const F2: Self = Self(13);
    pub const G2: Self = Self(14);
    pub const H2: Self = Self(15);

    pub const A3: Self = Self(16);
    pub const B3: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const D3: Self = Self(19);
    pub const E3: Self = Self(20);
    pub const F3: Self = Self(21);
    pub const G3: Self = Self(22);
    pub const H3: Self = Self(23);

    pub const A4: Self = Self(24);
    pub const B4: Self = Self(25);
    pub const C4: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const E4: Self = Self(28);
    pub const F4: Self = Self(29);
    pub const G4: Self = Self(30);
    pub const H4: Self = Self(31);

    pub const A5: Self = Self(32);
    pub const B5: Self = Self(33);
    pub const C5: Self = Self(34);
    pub const D5: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const F5: Self = Self(37);
    pub const G5: Self = Self(38);
    pub const H5: Self = Self(39);

    pub const A6: Self = Self(40);
    pub const B6: Self = Self(41);
    pub const C6: Self = Self(42);
    pub const D6: Self = Self(43);
    pub const E6: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const G6: Self = Self(46);
    pub const H6: Self = Self(47);

    pub const A7: Self = Self(48);
    pub const B7: Self = Self(49);
    pub const C7: Self = Self(50);
    pub const D7: Self = Self(51);
    pub const E7: Self = Self(52);
    pub const F7: Self = Self(53);
    pub const G7: Self = Self(54);
    pub const H7: Self = Self(55);

    pub const A8: Self = Self(56);
    pub const B8: Self = Self(57);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);

    #[inline(always)]
    pub fn from_file_rank(file: File, rank: Rank) -> Self {
        Self(rank as u8 * 8 + file as u8)
    }

    #[inline(always)]
    pub fn file(self) -> File {
        File::from_u8(self.0 % 8)
    }

    #[inline(always)]
    pub fn rank(self) -> Rank {
        Rank::from_u8(self.0 / 8)
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Flip rank (mirrors the square vertically, e.g. A1 ↔ A8).
    #[inline(always)]
    pub fn flip_rank(self) -> Self {
        Self(self.0 ^ 56)
    }

    /// Chebyshev (king) distance between two squares.
    #[inline(always)]
    pub fn chebyshev_distance(self, other: Self) -> u8 {
        let df = (self.file() as i8 - other.file() as i8).unsigned_abs();
        let dr = (self.rank() as i8 - other.rank() as i8).unsigned_abs();
        df.max(dr)
    }

    /// Parse from algebraic notation (e.g. "e4").
    pub fn from_algebraic(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let file_char = chars.next()?;
        let rank_char = chars.next()?;
        if chars.next().is_some() {
            return None;
        }
        let file = match file_char {
            'a' => File::A,
            'b' => File::B,
            'c' => File::C,
            'd' => File::D,
            'e' => File::E,
            'f' => File::F,
            'g' => File::G,
            'h' => File::H,
            _ => return None,
        };
        let rank = match rank_char {
            '1' => Rank::R1,
            '2' => Rank::R2,
            '3' => Rank::R3,
            '4' => Rank::R4,
            '5' => Rank::R5,
            '6' => Rank::R6,
            '7' => Rank::R7,
            '8' => Rank::R8,
            _ => return None,
        };
        Some(Self::from_file_rank(file, rank))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = b"abcdefgh"[self.file() as usize] as char;
        let rank = b"12345678"[self.rank() as usize] as char;
        write!(f, "{file}{rank}")
    }
}

// -----------------------------------------------------------------------
// File helpers
// -----------------------------------------------------------------------

impl File {
    #[inline(always)]
    pub fn from_u8(v: u8) -> Self {
        debug_assert!(v < 8);
        // SAFETY: File is repr(u8) with variants 0-7
        unsafe { std::mem::transmute(v) }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'a' => Some(Self::A),
            'b' => Some(Self::B),
            'c' => Some(Self::C),
            'd' => Some(Self::D),
            'e' => Some(Self::E),
            'f' => Some(Self::F),
            'g' => Some(Self::G),
            'h' => Some(Self::H),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------
// Rank helpers
// -----------------------------------------------------------------------

impl Rank {
    #[inline(always)]
    pub fn from_u8(v: u8) -> Self {
        debug_assert!(v < 8);
        unsafe { std::mem::transmute(v) }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::R1),
            '2' => Some(Self::R2),
            '3' => Some(Self::R3),
            '4' => Some(Self::R4),
            '5' => Some(Self::R5),
            '6' => Some(Self::R6),
            '7' => Some(Self::R7),
            '8' => Some(Self::R8),
            _ => None,
        }
    }
}
