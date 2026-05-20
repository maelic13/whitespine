use std::fmt;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, Shr, Sub,
};

use super::square::Square;

/// A 64-bit set of squares. LSB = A1 (index 0), MSB = H8 (index 63).
#[derive(Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u64::MAX);

    // Rank masks
    pub const RANK_1: Self = Self(0x0000_0000_0000_00FF);
    pub const RANK_2: Self = Self(0x0000_0000_0000_FF00);
    pub const RANK_3: Self = Self(0x0000_0000_00FF_0000);
    pub const RANK_4: Self = Self(0x0000_0000_FF00_0000);
    pub const RANK_5: Self = Self(0x0000_00FF_0000_0000);
    pub const RANK_6: Self = Self(0x0000_FF00_0000_0000);
    pub const RANK_7: Self = Self(0x00FF_0000_0000_0000);
    pub const RANK_8: Self = Self(0xFF00_0000_0000_0000);

    // File masks
    pub const FILE_A: Self = Self(0x0101_0101_0101_0101);
    pub const FILE_B: Self = Self(0x0202_0202_0202_0202);
    pub const FILE_G: Self = Self(0x4040_4040_4040_4040);
    pub const FILE_H: Self = Self(0x8080_8080_8080_8080);
    pub const NOT_FILE_A: Self = Self(!0x0101_0101_0101_0101);
    pub const NOT_FILE_H: Self = Self(!0x8080_8080_8080_8080);

    /// Light squares (a1 is dark, b1 is light in standard orientation)
    pub const LIGHT_SQUARES: Self = Self(0xAA55_AA55_AA55_AA55);
    pub const DARK_SQUARES: Self = Self(0x55AA_55AA_55AA_55AA);

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn any(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns true if more than one bit is set.
    #[inline(always)]
    pub const fn more_than_one(self) -> bool {
        (self.0 & self.0.wrapping_sub(1)) != 0
    }

    /// Index of the least significant set bit (square index).
    #[inline(always)]
    pub fn lsb(self) -> Square {
        debug_assert!(self.0 != 0);
        Square(self.0.trailing_zeros() as u8)
    }

    /// Index of the most significant set bit (square index).
    #[inline(always)]
    pub fn msb(self) -> Square {
        debug_assert!(self.0 != 0);
        Square(63 - self.0.leading_zeros() as u8)
    }

    /// Remove and return the LSB square (used in iteration).
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Square {
        let sq = self.lsb();
        self.0 &= self.0 - 1;
        sq
    }

    // -----------------------------------------------------------------------
    // Directional shifts (no-wrap versions via masking)
    // -----------------------------------------------------------------------

    #[inline(always)]
    pub const fn north(self) -> Self {
        Self(self.0 << 8)
    }

    #[inline(always)]
    pub const fn south(self) -> Self {
        Self(self.0 >> 8)
    }

    #[inline(always)]
    pub const fn east(self) -> Self {
        // mask off file H to prevent wrapping
        Self((self.0 & !0x8080_8080_8080_8080) << 1)
    }

    #[inline(always)]
    pub const fn west(self) -> Self {
        // mask off file A
        Self((self.0 & !0x0101_0101_0101_0101) >> 1)
    }

    #[inline(always)]
    pub const fn north_east(self) -> Self {
        Self((self.0 & !0x8080_8080_8080_8080) << 9)
    }

    #[inline(always)]
    pub const fn north_west(self) -> Self {
        Self((self.0 & !0x0101_0101_0101_0101) << 7)
    }

    #[inline(always)]
    pub const fn south_east(self) -> Self {
        Self((self.0 & !0x8080_8080_8080_8080) >> 7)
    }

    #[inline(always)]
    pub const fn south_west(self) -> Self {
        Self((self.0 & !0x0101_0101_0101_0101) >> 9)
    }
}

// -----------------------------------------------------------------------
// Iterator — yields squares from LSB to MSB
// -----------------------------------------------------------------------

impl Iterator for Bitboard {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(self.pop_lsb())
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for Bitboard {}

// -----------------------------------------------------------------------
// Operator impls
// -----------------------------------------------------------------------

impl BitAnd for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitXor for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl Not for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        Self(!self.0)
    }
}
impl BitAndAssign for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl BitXorAssign for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
impl Sub for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }
}
impl Mul for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }
}
impl Shl<u32> for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: u32) -> Self {
        Self(self.0 << rhs)
    }
}
impl Shr<u32> for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: u32) -> Self {
        Self(self.0 >> rhs)
    }
}

impl From<Square> for Bitboard {
    #[inline(always)]
    fn from(sq: Square) -> Self {
        Self(1u64 << sq.0)
    }
}

// -----------------------------------------------------------------------
// Display / Debug
// -----------------------------------------------------------------------

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bitboard(0x{:016X})", self.0)?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let bit = (self.0 >> (rank * 8 + file)) & 1;
                write!(f, "{}", if bit == 1 { "1 " } else { ". " })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
