use std::mem::size_of;
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU64, Ordering},
};

use crate::board::Move;
use crate::eval::MATE_SCORE;

const MAX_PLY: i32 = 128;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact = 1,
    Upper = 2,
    Lower = 3,
}

impl Bound {
    #[inline(always)]
    fn from_bits(bits: u8) -> Option<Self> {
        match bits & 3 {
            1 => Some(Self::Exact),
            2 => Some(Self::Upper),
            3 => Some(Self::Lower),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct TtEntry {
    key16: u16,
    pub score: i16,
    pub static_eval: i16,
    pub mv: u16,
    pub depth: i8,
    flag_age: u8,
}

impl TtEntry {
    #[inline(always)]
    pub fn bound(self) -> Option<Bound> {
        Bound::from_bits(self.flag_age)
    }

    #[inline(always)]
    pub fn best_move(self) -> Option<Move> {
        (self.mv != 0).then_some(Move(self.mv))
    }
}

#[repr(align(32))]
#[derive(Copy, Clone, Default)]
struct LocalCluster {
    entries: [TtEntry; 3],
    _padding: [u8; 2],
}

#[derive(Clone)]
struct LocalTable {
    clusters: Vec<LocalCluster>,
    mask: usize,
    age: u8,
}

struct AtomicTtEntry {
    key_xor_data: AtomicU64,
    data: AtomicU64,
}

impl Default for AtomicTtEntry {
    fn default() -> Self {
        Self {
            key_xor_data: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }
}

impl AtomicTtEntry {
    #[inline(always)]
    fn load(&self, key: u64) -> Option<TtEntry> {
        let data = self.data.load(Ordering::Relaxed);
        let stored_key = self.key_xor_data.load(Ordering::Relaxed) ^ data;
        if stored_key != key {
            return None;
        }
        Self::unpack(stored_key, data)
    }

    #[inline(always)]
    fn load_any(&self) -> Option<(u64, TtEntry)> {
        let data = self.data.load(Ordering::Relaxed);
        let stored_key = self.key_xor_data.load(Ordering::Relaxed) ^ data;
        Self::unpack(stored_key, data).map(|entry| (stored_key, entry))
    }

    #[inline(always)]
    fn unpack(key: u64, data: u64) -> Option<TtEntry> {
        let flag_age = (data >> 56) as u8;
        Bound::from_bits(flag_age)?;

        Some(TtEntry {
            key16: (key >> 48) as u16,
            score: (data as u16) as i16,
            static_eval: ((data >> 16) as u16) as i16,
            mv: (data >> 32) as u16,
            depth: ((data >> 48) as u8) as i8,
            flag_age,
        })
    }

    #[inline(always)]
    fn store(&self, key: u64, entry: TtEntry) {
        let data = entry.score as u16 as u64
            | ((entry.static_eval as u16 as u64) << 16)
            | ((entry.mv as u64) << 32)
            | ((entry.depth as u8 as u64) << 48)
            | ((entry.flag_age as u64) << 56);

        self.data.store(data, Ordering::Relaxed);
        self.key_xor_data.store(key ^ data, Ordering::Relaxed);
    }

    #[inline(always)]
    fn clear(&self) {
        self.data.store(0, Ordering::Relaxed);
        self.key_xor_data.store(0, Ordering::Relaxed);
    }
}

#[repr(align(64))]
struct SharedCluster {
    entries: [AtomicTtEntry; 3],
}

impl Default for SharedCluster {
    fn default() -> Self {
        Self {
            entries: std::array::from_fn(|_| AtomicTtEntry::default()),
        }
    }
}

struct SharedTable {
    clusters: Box<[SharedCluster]>,
    mask: usize,
    age: AtomicU8,
}

#[derive(Clone)]
enum TtStorage {
    Local(LocalTable),
    Shared(Arc<SharedTable>),
}

#[derive(Clone)]
pub struct TranspositionTable {
    storage: TtStorage,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(64)
    }
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        Self {
            storage: TtStorage::Local(new_local_table(mb).unwrap_or_else(|| {
                new_local_table(1).expect("1 MiB transposition table must allocate")
            })),
        }
    }

    pub fn resize(&mut self, mb: usize) -> bool {
        if let Some(table) = new_local_table(mb) {
            self.storage = TtStorage::Local(table);
            true
        } else {
            false
        }
    }

    pub fn ensure_local(&mut self, mb: usize) -> bool {
        if !matches!(self.storage, TtStorage::Local(_)) {
            if let Some(table) = new_local_table(mb) {
                self.storage = TtStorage::Local(table);
            } else {
                return false;
            }
        }
        true
    }

    pub fn make_shared(&mut self) {
        if let TtStorage::Local(local) = &self.storage {
            self.storage = TtStorage::Shared(Arc::new(shared_from_local(local)));
        }
    }

    pub fn clear(&mut self) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                table.clusters.fill(LocalCluster::default());
                table.age = 0;
            }
            TtStorage::Shared(table) => {
                for cluster in table.clusters.iter() {
                    for entry in &cluster.entries {
                        entry.clear();
                    }
                }
                table.age.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn new_search(&mut self) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                table.age = table.age.wrapping_add(4) & 0xFC;
            }
            TtStorage::Shared(table) => {
                let age = table.age.load(Ordering::Relaxed);
                table
                    .age
                    .store(age.wrapping_add(4) & 0xFC, Ordering::Relaxed);
            }
        }
    }

    #[inline(always)]
    pub fn probe(&self, key: u64) -> Option<TtEntry> {
        match &self.storage {
            TtStorage::Local(table) => probe_local(table, key),
            TtStorage::Shared(table) => probe_shared(table, key),
        }
    }

    #[inline(always)]
    pub fn store(
        &mut self,
        key: u64,
        depth: i32,
        score: i32,
        bound: Bound,
        mv: Move,
        ply: usize,
        static_eval: i32,
    ) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                store_local(table, key, depth, score, bound, mv, ply, static_eval);
            }
            TtStorage::Shared(table) => {
                store_shared(table, key, depth, score, bound, mv, ply, static_eval);
            }
        }
    }

    pub fn hashfull(&self) -> usize {
        match &self.storage {
            TtStorage::Local(table) => {
                let sample = table.clusters.len().min(334);
                if sample == 0 {
                    return 0;
                }
                let used = table
                    .clusters
                    .iter()
                    .take(sample)
                    .flat_map(|cluster| cluster.entries)
                    .filter(|entry| entry.bound().is_some())
                    .count();
                used * 1000 / (sample * 3)
            }
            TtStorage::Shared(table) => {
                let sample = table.clusters.len().min(334);
                if sample == 0 {
                    return 0;
                }
                let used = table
                    .clusters
                    .iter()
                    .take(sample)
                    .flat_map(|cluster| cluster.entries.iter())
                    .filter_map(AtomicTtEntry::load_any)
                    .count();
                used * 1000 / (sample * 3)
            }
        }
    }
}

pub fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score >= MATE_SCORE - MAX_PLY {
        score + ply as i32
    } else if score <= -MATE_SCORE + MAX_PLY {
        score - ply as i32
    } else {
        score
    }
}

pub fn score_from_tt(score: i32, ply: usize) -> i32 {
    if score >= MATE_SCORE - MAX_PLY {
        score - ply as i32
    } else if score <= -MATE_SCORE + MAX_PLY {
        score + ply as i32
    } else {
        score
    }
}

fn new_local_table(mb: usize) -> Option<LocalTable> {
    let power = cluster_count::<LocalCluster>(mb);
    let mut clusters = Vec::new();
    clusters.try_reserve_exact(power).ok()?;
    clusters.resize(power, LocalCluster::default());
    LocalTable {
        clusters,
        mask: power - 1,
        age: 0,
    }
    .into()
}

fn shared_from_local(local: &LocalTable) -> SharedTable {
    let clusters = (0..local.clusters.len())
        .map(|_| SharedCluster::default())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    SharedTable {
        clusters,
        mask: local.mask,
        age: AtomicU8::new(local.age),
    }
}

fn cluster_count<T>(mb: usize) -> usize {
    let bytes = mb.max(1).saturating_mul(1024).saturating_mul(1024);
    let count = (bytes / size_of::<T>()).max(1);
    let mut power = 1usize;
    while power <= count / 2 {
        power *= 2;
    }
    power
}

#[inline(always)]
fn probe_local(table: &LocalTable, key: u64) -> Option<TtEntry> {
    let key16 = (key >> 48) as u16;
    table.clusters[key as usize & table.mask]
        .entries
        .iter()
        .copied()
        .find(|entry| entry.key16 == key16 && entry.bound().is_some())
}

#[inline(always)]
fn probe_shared(table: &SharedTable, key: u64) -> Option<TtEntry> {
    table.clusters[key as usize & table.mask]
        .entries
        .iter()
        .find_map(|slot| slot.load(key))
}

#[inline(always)]
fn store_local(
    table: &mut LocalTable,
    key: u64,
    depth: i32,
    score: i32,
    bound: Bound,
    mv: Move,
    ply: usize,
    static_eval: i32,
) {
    let key16 = (key >> 48) as u16;
    let cluster = &mut table.clusters[key as usize & table.mask];

    let mut replace_index = 0usize;
    let mut replace_quality = i32::MAX;
    for (index, entry) in cluster.entries.iter().enumerate() {
        if entry.key16 == key16 {
            replace_index = index;
            break;
        }
        let quality = entry_quality(*entry, table.age);
        if quality < replace_quality {
            replace_quality = quality;
            replace_index = index;
        }
    }

    let replace = &mut cluster.entries[replace_index];
    if replace.key16 == key16
        && bound != Bound::Exact
        && depth < replace.depth as i32 - 3
        && (replace.flag_age & 0xFC) == table.age
    {
        return;
    }

    let stored_move = if mv.is_null() && replace.key16 == key16 {
        replace.mv
    } else {
        mv.0
    };

    *replace = make_entry(
        key16,
        depth,
        score,
        bound,
        stored_move,
        ply,
        static_eval,
        table.age,
    );
}

#[inline(always)]
fn store_shared(
    table: &SharedTable,
    key: u64,
    depth: i32,
    score: i32,
    bound: Bound,
    mv: Move,
    ply: usize,
    static_eval: i32,
) {
    let age = table.age.load(Ordering::Relaxed);
    let key16 = (key >> 48) as u16;
    let cluster = &table.clusters[key as usize & table.mask];

    let mut replace_index = 0usize;
    let mut replace_quality = i32::MAX;
    let mut replace_entry = TtEntry::default();
    let mut replace_key = 0u64;
    for (index, slot) in cluster.entries.iter().enumerate() {
        let (entry_key, entry) = slot.load_any().unwrap_or_default();
        if entry_key == key && entry.bound().is_some() {
            replace_index = index;
            replace_entry = entry;
            replace_key = entry_key;
            break;
        }
        let quality = entry_quality(entry, age);
        if quality < replace_quality {
            replace_quality = quality;
            replace_index = index;
            replace_entry = entry;
            replace_key = entry_key;
        }
    }

    if replace_key == key
        && bound != Bound::Exact
        && depth < replace_entry.depth as i32 - 3
        && (replace_entry.flag_age & 0xFC) == age
    {
        return;
    }

    let stored_move = if mv.is_null() && replace_key == key {
        replace_entry.mv
    } else {
        mv.0
    };

    cluster.entries[replace_index].store(
        key,
        make_entry(
            key16,
            depth,
            score,
            bound,
            stored_move,
            ply,
            static_eval,
            age,
        ),
    );
}

#[inline(always)]
fn make_entry(
    key16: u16,
    depth: i32,
    score: i32,
    bound: Bound,
    mv: u16,
    ply: usize,
    static_eval: i32,
    age: u8,
) -> TtEntry {
    TtEntry {
        key16,
        score: score_to_tt(score, ply).clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        static_eval: static_eval.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        mv,
        depth: depth.clamp(-1, i8::MAX as i32) as i8,
        flag_age: age | bound as u8,
    }
}

#[inline(always)]
fn entry_quality(entry: TtEntry, age: u8) -> i32 {
    if entry.bound().is_none() {
        return i32::MIN;
    }
    let age_delta = age.wrapping_sub(entry.flag_age & 0xFC) & 0xFC;
    entry.depth as i32 - age_delta as i32 / 2
}
