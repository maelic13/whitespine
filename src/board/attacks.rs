/// Attack tables for all piece types.
///
/// Non-sliding pieces (pawn, knight, king) are stored as simple lookup arrays.
/// Sliding pieces (bishop, rook, queen) use magic bitboards, initialized at
/// startup via `LazyLock`.
use std::sync::LazyLock;

use super::bitboard::Bitboard;
use super::piece::Color;
use super::square::Square;

// -----------------------------------------------------------------------
// Public attack accessors
// -----------------------------------------------------------------------

/// All attack tables, initialized once.
pub struct AttackTables {
    pub pawn_attacks: [[Bitboard; 64]; 2],
    pub knight_attacks: [Bitboard; 64],
    pub king_attacks: [Bitboard; 64],
    bishop: [MagicEntry; 64],
    rook: [MagicEntry; 64],
    bishop_table: Vec<Bitboard>,
    rook_table: Vec<Bitboard>,
}

pub static ATTACKS: LazyLock<AttackTables> = LazyLock::new(AttackTables::init);

struct MagicEntry {
    mask: u64,
    magic: u64,
    shift: u32,
    offset: usize,
}

impl AttackTables {
    // -----------------------------------------------------------------------
    // Public accessors
    // -----------------------------------------------------------------------

    #[inline(always)]
    pub fn pawn(&self, color: Color, sq: Square) -> Bitboard {
        self.pawn_attacks[color as usize][sq.index()]
    }

    #[inline(always)]
    pub fn knight(&self, sq: Square) -> Bitboard {
        self.knight_attacks[sq.index()]
    }

    #[inline(always)]
    pub fn king(&self, sq: Square) -> Bitboard {
        self.king_attacks[sq.index()]
    }

    #[inline(always)]
    pub fn bishop(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let e = &self.bishop[sq.index()];
        let idx = e.offset + (((occ.0 & e.mask).wrapping_mul(e.magic)) >> e.shift) as usize;
        self.bishop_table[idx]
    }

    #[inline(always)]
    pub fn rook(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let e = &self.rook[sq.index()];
        let idx = e.offset + (((occ.0 & e.mask).wrapping_mul(e.magic)) >> e.shift) as usize;
        self.rook_table[idx]
    }

    #[inline(always)]
    pub fn queen(&self, sq: Square, occ: Bitboard) -> Bitboard {
        self.bishop(sq, occ) | self.rook(sq, occ)
    }

    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    fn init() -> Self {
        let pawn_attacks = Self::init_pawn_attacks();
        let knight_attacks = Self::init_knight_attacks();
        let king_attacks = Self::init_king_attacks();

        // Bishop magic tables
        let mut bishop_entries: [MagicEntry; 64] = std::array::from_fn(|_| MagicEntry {
            mask: 0,
            magic: 0,
            shift: 0,
            offset: 0,
        });
        let mut bishop_table: Vec<Bitboard> = Vec::new();
        let mut rng = Rng::new(0x1234_5678_9ABC_DEF0);
        for sq in 0..64 {
            let sq = Square(sq as u8);
            let mask = bishop_mask(sq);
            let n = mask.count_ones() as u32;
            let shift = 64 - n;
            let size = 1usize << n;
            let offset = bishop_table.len();
            bishop_table.resize(offset + size, Bitboard::EMPTY);
            let magic = find_magic(
                mask,
                shift,
                false,
                sq,
                &mut rng,
                &mut bishop_table[offset..],
            );
            bishop_entries[sq.index()] = MagicEntry {
                mask,
                magic,
                shift,
                offset,
            };
        }

        // Rook magic tables
        let mut rook_entries: [MagicEntry; 64] = std::array::from_fn(|_| MagicEntry {
            mask: 0,
            magic: 0,
            shift: 0,
            offset: 0,
        });
        let mut rook_table: Vec<Bitboard> = Vec::new();
        for sq in 0..64 {
            let sq = Square(sq as u8);
            let mask = rook_mask(sq);
            let n = mask.count_ones() as u32;
            let shift = 64 - n;
            let size = 1usize << n;
            let offset = rook_table.len();
            rook_table.resize(offset + size, Bitboard::EMPTY);
            let magic = find_magic(mask, shift, true, sq, &mut rng, &mut rook_table[offset..]);
            rook_entries[sq.index()] = MagicEntry {
                mask,
                magic,
                shift,
                offset,
            };
        }

        Self {
            pawn_attacks,
            knight_attacks,
            king_attacks,
            bishop: bishop_entries,
            rook: rook_entries,
            bishop_table,
            rook_table,
        }
    }

    fn init_pawn_attacks() -> [[Bitboard; 64]; 2] {
        let mut table = [[Bitboard::EMPTY; 64]; 2];
        for s in 0..64u8 {
            let sq = Square(s);
            let bb = Bitboard::from(sq);
            // White pawns attack north-east and north-west
            table[Color::White as usize][sq.index()] = bb.north_east() | bb.north_west();
            // Black pawns attack south-east and south-west
            table[Color::Black as usize][sq.index()] = bb.south_east() | bb.south_west();
        }
        table
    }

    fn init_knight_attacks() -> [Bitboard; 64] {
        let mut table = [Bitboard::EMPTY; 64];
        for s in 0..64u8 {
            let sq = Square(s);
            let bb = Bitboard::from(sq);
            // Two-square jumps: (±1, ±2) and (±2, ±1)
            let h1 = bb.east() | bb.west();
            let h2 = bb.east().east() | bb.west().west();
            table[sq.index()] = h1.north().north() | h1.south().south() | h2.north() | h2.south();
        }
        table
    }

    fn init_king_attacks() -> [Bitboard; 64] {
        let mut table = [Bitboard::EMPTY; 64];
        for s in 0..64u8 {
            let sq = Square(s);
            let bb = Bitboard::from(sq);
            table[sq.index()] = bb.north()
                | bb.south()
                | bb.east()
                | bb.west()
                | bb.north_east()
                | bb.north_west()
                | bb.south_east()
                | bb.south_west();
        }
        table
    }
}

// -----------------------------------------------------------------------
// Slow (reference) attack generators — used only during magic init
// -----------------------------------------------------------------------

/// Rook relevant occupancy mask (excludes edges).
fn rook_mask(sq: Square) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut mask = 0u64;
    for i in (r + 1)..7 {
        mask |= 1u64 << (i * 8 + f);
    }
    for i in 1..r {
        mask |= 1u64 << (i * 8 + f);
    }
    for i in (f + 1)..7 {
        mask |= 1u64 << (r * 8 + i);
    }
    for i in 1..f {
        mask |= 1u64 << (r * 8 + i);
    }
    mask
}

/// Bishop relevant occupancy mask (excludes edges).
fn bishop_mask(sq: Square) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut mask = 0u64;
    for i in 1..8i32 {
        let nr = r as i32 + i;
        let nf = f as i32 + i;
        if nr >= 7 || nf >= 7 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    for i in 1..8i32 {
        let nr = r as i32 + i;
        let nf = f as i32 - i;
        if nr >= 7 || nf <= 0 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    for i in 1..8i32 {
        let nr = r as i32 - i;
        let nf = f as i32 + i;
        if nr <= 0 || nf >= 7 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    for i in 1..8i32 {
        let nr = r as i32 - i;
        let nf = f as i32 - i;
        if nr <= 0 || nf <= 0 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    mask
}

fn rook_attacks_slow(sq: Square, occ: u64) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut att = 0u64;
    for i in (r + 1)..8 {
        att |= 1u64 << (i * 8 + f);
        if (occ >> (i * 8 + f)) & 1 != 0 {
            break;
        }
    }
    for i in (0..r).rev() {
        att |= 1u64 << (i * 8 + f);
        if (occ >> (i * 8 + f)) & 1 != 0 {
            break;
        }
    }
    for i in (f + 1)..8 {
        att |= 1u64 << (r * 8 + i);
        if (occ >> (r * 8 + i)) & 1 != 0 {
            break;
        }
    }
    for i in (0..f).rev() {
        att |= 1u64 << (r * 8 + i);
        if (occ >> (r * 8 + i)) & 1 != 0 {
            break;
        }
    }
    att
}

fn bishop_attacks_slow(sq: Square, occ: u64) -> u64 {
    let r = sq.0 as i32 / 8;
    let f = sq.0 as i32 % 8;
    let mut att = 0u64;
    for i in 1..8 {
        let (nr, nf) = (r + i, f + i);
        if nr >= 8 || nf >= 8 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    for i in 1..8 {
        let (nr, nf) = (r + i, f - i);
        if nr >= 8 || nf < 0 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    for i in 1..8 {
        let (nr, nf) = (r - i, f + i);
        if nr < 0 || nf >= 8 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    for i in 1..8 {
        let (nr, nf) = (r - i, f - i);
        if nr < 0 || nf < 0 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    att
}

// -----------------------------------------------------------------------
// Magic finding
// -----------------------------------------------------------------------

/// Find a magic number for `sq` with the given `mask` / `shift`.
/// Fills `table[0..size]` with the correct attack bitboards on success.
fn find_magic(
    mask: u64,
    shift: u32,
    is_rook: bool,
    sq: Square,
    rng: &mut Rng,
    table: &mut [Bitboard],
) -> u64 {
    let n = mask.count_ones() as usize;
    let size = 1usize << n;
    debug_assert_eq!(table.len(), size);

    // Enumerate all subsets of `mask` via carry-rippler and precompute attacks.
    let mut occs = vec![0u64; size];
    let mut atts = vec![0u64; size];
    let mut occ = 0u64;
    let mut i = 0;
    loop {
        occs[i] = occ;
        atts[i] = if is_rook {
            rook_attacks_slow(sq, occ)
        } else {
            bishop_attacks_slow(sq, occ)
        };
        i += 1;
        occ = occ.wrapping_sub(mask) & mask;
        if occ == 0 {
            break;
        }
    }

    // Try random sparse candidates until we find a valid magic.
    loop {
        let magic = rng.sparse();
        // Quick reject: upper byte of (mask * magic) should have enough bits set.
        if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 {
            continue;
        }

        // Reset table
        for t in table.iter_mut() {
            *t = Bitboard::EMPTY;
        }

        let mut ok = true;
        for j in 0..size {
            let idx = ((occs[j].wrapping_mul(magic)) >> shift) as usize;
            if table[idx].is_empty() {
                table[idx] = Bitboard(atts[j]);
            } else if table[idx].0 != atts[j] {
                ok = false;
                break;
            }
        }

        if ok {
            return magic;
        }
    }
}

// -----------------------------------------------------------------------
// splitmix64 PRNG (sparse variant) — same as basilisk
// -----------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    #[inline]
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Sparse 64-bit value (AND of three randoms — biased toward few set bits,
    /// which gives good magic candidates).
    #[inline]
    fn sparse(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}
