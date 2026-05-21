use crate::board::{Move, Piece};

pub const HISTORY_MAX: i32 = 16_384;
pub const CAP_HISTORY_MAX: i32 = 16_384;
pub const CORR_SIZE: usize = 16_384;
pub const CONT_SIZE: usize = 6 * 64 * 6 * 64;

#[derive(Copy, Clone, Default)]
pub(crate) struct ScoredMove {
    pub mv: Move,
    pub score: i32,
    pub see: i16,
}

pub(crate) struct ScoredMoveList {
    moves: [ScoredMove; 256],
    len: usize,
}

impl ScoredMoveList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            moves: [ScoredMove::default(); 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, mv: Move, score: i32, see: i32) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len] = ScoredMove {
            mv,
            score,
            see: see.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        };
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [ScoredMove] {
        &mut self.moves[..self.len]
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BadCapture {
    pub attacker: Piece,
    pub to: usize,
    pub captured: Option<Piece>,
}

pub(crate) struct BadCaptureList {
    items: [BadCapture; 256],
    len: usize,
}

impl BadCaptureList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            items: [BadCapture {
                attacker: Piece::Pawn,
                to: 0,
                captured: None,
            }; 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, attacker: Piece, to: usize, captured: Option<Piece>) {
        debug_assert!(self.len < self.items.len());
        self.items[self.len] = BadCapture {
            attacker,
            to,
            captured,
        };
        self.len += 1;
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[BadCapture] {
        &self.items[..self.len]
    }
}

pub(crate) fn pick_next(moves: &mut [ScoredMove], index: usize) -> ScoredMove {
    let mut best = index;
    for current in index + 1..moves.len() {
        if moves[current].score > moves[best].score {
            best = current;
        }
    }
    moves.swap(index, best);
    moves[index]
}

pub(crate) fn diversify_root_scores(moves: &mut [ScoredMove], offset: usize) {
    moves.sort_unstable_by(|left, right| right.score.cmp(&left.score));
    if offset < moves.len() {
        moves[offset].score = moves[0].score.saturating_add(1_000_000);
    }
}

pub(crate) fn update_hist_entry(entry: &mut i16, bonus: i32, max_value: i32) {
    let current = *entry as i32;
    let updated = current + bonus - current * bonus.abs() / max_value;
    *entry = updated.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
}

pub(crate) fn history_bonus(depth: i32) -> i32 {
    (depth * depth + 2 * depth).min(1_200)
}

pub(crate) fn cont_index(prev_piece: usize, prev_to: usize, piece: usize, to: usize) -> usize {
    (((prev_piece * 64 + prev_to) * 6 + piece) * 64 + to).min(CONT_SIZE - 1)
}
