/// Zobrist hash tables — initialized at startup via `LazyLock`.
///
/// Keys are generated with splitmix64, matching basilisk's seed for
/// reproducibility across restarts.
use std::sync::LazyLock;

use super::piece::{CastlingRights, Color, Piece};
use super::square::{File, Square};

pub struct ZobristKeys {
    /// `piece_keys[color][piece][square]`
    pub piece_keys: [[[u64; 64]; 6]; 2],
    /// XOR in when it is black's turn to move.
    pub side_key: u64,
    /// `castling_keys[rights.0 as usize]` — 16 entries
    pub castling_keys: [u64; 16],
    /// `ep_keys[file]` — XOR in when en passant is available on that file
    pub ep_keys: [u64; 8],
}

pub static ZOBRIST: LazyLock<ZobristKeys> = LazyLock::new(ZobristKeys::init);

impl ZobristKeys {
    fn init() -> Self {
        let mut state = 0xDEAD_BEEF_CAFE_BABE_u64;

        let mut piece_keys = [[[0u64; 64]; 6]; 2];
        for c in 0..2 {
            for pt in 0..6 {
                for sq in 0..64 {
                    piece_keys[c][pt][sq] = sm64_next(&mut state);
                }
            }
        }

        let side_key = sm64_next(&mut state);

        let mut castling_keys = [0u64; 16];
        for k in &mut castling_keys {
            *k = sm64_next(&mut state);
        }

        let mut ep_keys = [0u64; 8];
        for k in &mut ep_keys {
            *k = sm64_next(&mut state);
        }

        Self {
            piece_keys,
            side_key,
            castling_keys,
            ep_keys,
        }
    }

    #[inline(always)]
    pub fn piece(&self, color: Color, piece: Piece, sq: Square) -> u64 {
        self.piece_keys[color as usize][piece as usize][sq.index()]
    }

    #[inline(always)]
    pub fn side(&self) -> u64 {
        self.side_key
    }

    #[inline(always)]
    pub fn castling(&self, rights: CastlingRights) -> u64 {
        self.castling_keys[rights.0 as usize]
    }

    #[inline(always)]
    pub fn ep(&self, file: File) -> u64 {
        self.ep_keys[file as usize]
    }
}

/// splitmix64 PRNG — same algorithm as basilisk.
#[inline(always)]
fn sm64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
