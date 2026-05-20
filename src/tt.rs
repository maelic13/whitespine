//! Transposition table (TT) - 3-entry clusters, 32-byte aligned.
use crate::board::moves::Move;

pub const MATE_SCORE: i32 = 32000;
pub const INF_EVAL: i32 = 32001;
pub const VALUE_NONE: i32 = 32002;
pub const MAX_PLY: usize = 128;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum TTFlag {
    None = 0,
    Exact = 1,
    Alpha = 2,
    Beta = 3,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TTEntry {
    pub key16: u16,
    pub score: i16,
    pub static_eval: i16,
    pub mv: u16,
    pub depth: i8,
    pub flag_age: u8,
}

impl TTEntry {
    pub fn flag(&self) -> TTFlag {
        match self.flag_age & 3 {
            1 => TTFlag::Exact,
            2 => TTFlag::Alpha,
            3 => TTFlag::Beta,
            _ => TTFlag::None,
        }
    }

    pub fn age(&self) -> u8 {
        self.flag_age >> 2
    }
}

#[repr(C, align(32))]
#[derive(Clone)]
struct TTCluster {
    entries: [TTEntry; 3],
    _pad: [u8; 2],
}

pub struct TranspositionTable {
    clusters: Vec<TTCluster>,
    mask: usize,
    age: u8,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        let bytes = mb.max(1) * 1024 * 1024;
        let n = (bytes / std::mem::size_of::<TTCluster>())
            .max(1)
            .next_power_of_two();
        // Zero-initialize: TTEntry with flag_age=0 → TTFlag::None (empty).
        // depth=0 and key16=0 are fine — probes check flag first.
        let clusters = vec![unsafe { std::mem::zeroed::<TTCluster>() }; n];
        Self {
            clusters,
            mask: n - 1,
            age: 0,
        }
    }

    pub fn resize(&mut self, mb: usize) {
        *self = Self::new(mb);
    }

    pub fn clear(&mut self) {
        // Safety: TTCluster is repr(C, align(32)), all-zeros is a valid empty state.
        unsafe {
            std::ptr::write_bytes(self.clusters.as_mut_ptr(), 0, self.clusters.len());
        }
        self.age = 0;
    }

    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(4);
    }

    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let idx = (hash as usize) & self.mask;
        let key16 = (hash >> 48) as u16;
        for entry in &self.clusters[idx].entries {
            if entry.key16 == key16 && entry.flag() != TTFlag::None {
                return Some(*entry);
            }
        }
        None
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: i32,
        flag: TTFlag,
        score: i32,
        static_eval: i32,
        mv: Move,
    ) {
        let idx = (hash as usize) & self.mask;
        let key16 = (hash >> 48) as u16;
        let cluster = &mut self.clusters[idx];

        let mut replace_idx = 0usize;
        let mut replace_val = i32::MAX;
        for (i, entry) in cluster.entries.iter().enumerate() {
            if entry.key16 == key16 || entry.flag() == TTFlag::None {
                replace_idx = i;
                break;
            }
            let age_penalty = if entry.age() == self.age >> 2 { 0 } else { 8 };
            let value = entry.depth as i32 - age_penalty;
            if value < replace_val {
                replace_val = value;
                replace_idx = i;
            }
        }

        let entry = &mut cluster.entries[replace_idx];
        if entry.key16 == key16 && flag != TTFlag::Exact && depth <= entry.depth as i32 {
            return;
        }

        entry.key16 = key16;
        entry.score = score.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        entry.static_eval = static_eval.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        entry.mv = mv.0;
        entry.depth = depth.clamp(-128, 127) as i8;
        entry.flag_age = (flag as u8) | (self.age & 0xFC);
    }

    pub fn hashfull(&self) -> usize {
        let n = 334.min(self.clusters.len());
        let filled: usize = self.clusters[..n]
            .iter()
            .map(|cluster| {
                cluster
                    .entries
                    .iter()
                    .filter(|entry| entry.flag() != TTFlag::None && entry.age() == self.age >> 2)
                    .count()
            })
            .sum();
        filled * 1000 / (n * 3)
    }

    pub fn score_to_tt(score: i32, ply: usize) -> i32 {
        if score >= MATE_SCORE - MAX_PLY as i32 {
            score + ply as i32
        } else if score <= -(MATE_SCORE - MAX_PLY as i32) {
            score - ply as i32
        } else {
            score
        }
    }

    pub fn score_from_tt(score: i32, ply: usize) -> i32 {
        if score >= MATE_SCORE - MAX_PLY as i32 {
            score - ply as i32
        } else if score <= -(MATE_SCORE - MAX_PLY as i32) {
            score + ply as i32
        } else {
            score
        }
    }
}
