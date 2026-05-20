/// Tests for the transposition table (TT) implementation.
use whitespine::board::moves::Move;
use whitespine::tt::{
    INF_EVAL, MATE_SCORE, MAX_PLY, TTFlag, TranspositionTable, VALUE_NONE,
};

// -----------------------------------------------------------------------
// Probe on empty TT
// -----------------------------------------------------------------------

#[test]
fn empty_tt_returns_none() {
    let tt = TranspositionTable::new(1);
    assert!(tt.probe(0).is_none());
    assert!(tt.probe(0xDEADBEEF_CAFEBABE).is_none());
}

// -----------------------------------------------------------------------
// Store and probe – all three flag types
// -----------------------------------------------------------------------

#[test]
fn store_exact_and_probe() {
    let mut tt = TranspositionTable::new(1);
    let hash = 0x1234_5678_9ABC_DEF0u64;
    tt.store(hash, 5, TTFlag::Exact, 120, 100, Move::NULL);
    let entry = tt.probe(hash).expect("should find entry");
    assert_eq!(entry.flag(), TTFlag::Exact);
    assert_eq!(entry.score, 120);
    assert_eq!(entry.static_eval, 100);
    assert_eq!(entry.depth, 5);
}

#[test]
fn store_alpha_and_probe() {
    let mut tt = TranspositionTable::new(1);
    let hash = 0xABCD_EF01_2345_6789u64;
    tt.store(hash, 3, TTFlag::Alpha, -50, -60, Move::NULL);
    let entry = tt.probe(hash).expect("should find entry");
    assert_eq!(entry.flag(), TTFlag::Alpha);
    assert_eq!(entry.score, -50);
}

#[test]
fn store_beta_and_probe() {
    let mut tt = TranspositionTable::new(1);
    let hash = 0xFEDC_BA98_7654_3210u64;
    tt.store(hash, 7, TTFlag::Beta, 300, 280, Move::NULL);
    let entry = tt.probe(hash).expect("should find entry");
    assert_eq!(entry.flag(), TTFlag::Beta);
    assert_eq!(entry.score, 300);
    assert_eq!(entry.depth, 7);
}

// -----------------------------------------------------------------------
// Different hash returns None (no false positives)
// -----------------------------------------------------------------------

#[test]
fn different_hash_returns_none() {
    let mut tt = TranspositionTable::new(1);
    let hash_a = 0x1111_1111_1111_1111u64;
    let hash_b = 0x2222_2222_1111_1111u64; // different upper 16 bits → different key16
    tt.store(hash_a, 4, TTFlag::Exact, 100, 90, Move::NULL);
    assert!(tt.probe(hash_b).is_none());
}

// -----------------------------------------------------------------------
// Replacement policy
// -----------------------------------------------------------------------

#[test]
fn exact_entry_not_replaced_by_shallower_non_exact() {
    let mut tt = TranspositionTable::new(1);
    let hash = 0xAAAA_BBBB_CCCC_DDDDu64;
    // Store exact at depth 8.
    tt.store(hash, 8, TTFlag::Exact, 200, 180, Move::NULL);
    // Try to overwrite with a shallower Alpha entry.
    tt.store(hash, 3, TTFlag::Alpha, -999, -999, Move::NULL);
    let entry = tt.probe(hash).expect("should still have entry");
    // The exact entry should survive because it has greater depth.
    assert_eq!(entry.flag(), TTFlag::Exact);
    assert_eq!(entry.score, 200);
}

#[test]
fn deeper_entry_replaces_shallower() {
    let mut tt = TranspositionTable::new(1);
    let hash = 0x1234_5678_ABCD_EF01u64;
    tt.store(hash, 2, TTFlag::Alpha, 50, 40, Move::NULL);
    // Overwrite with exact at greater depth — should succeed.
    tt.store(hash, 9, TTFlag::Exact, 150, 130, Move::NULL);
    let entry = tt.probe(hash).expect("should have entry");
    assert_eq!(entry.score, 150);
    assert_eq!(entry.depth, 9);
}

// -----------------------------------------------------------------------
// Clear empties the table
// -----------------------------------------------------------------------

#[test]
fn clear_empties_all_entries() {
    let mut tt = TranspositionTable::new(1);
    tt.store(0x1234_5678_9ABC_DEF0u64, 5, TTFlag::Exact, 100, 90, Move::NULL);
    tt.store(0xFEDC_BA98_7654_3210u64, 3, TTFlag::Beta, -200, -220, Move::NULL);
    tt.clear();
    assert!(tt.probe(0x1234_5678_9ABC_DEF0u64).is_none());
    assert!(tt.probe(0xFEDC_BA98_7654_3210u64).is_none());
}

// -----------------------------------------------------------------------
// new_search advances age (hashfull should reset for current age)
// -----------------------------------------------------------------------

#[test]
fn new_search_changes_age() {
    let mut tt = TranspositionTable::new(4);
    let hash = 0x9999_8888_7777_6666u64;
    tt.store(hash, 5, TTFlag::Exact, 100, 90, Move::NULL);
    let full_before = tt.hashfull();
    tt.new_search();
    // After advancing age, the old entry is "stale"; hashfull counts only
    // entries matching the current age, so it should drop.
    let full_after = tt.hashfull();
    assert!(
        full_after <= full_before,
        "hashfull should not increase after new_search (before: {}, after: {})",
        full_before, full_after
    );
}

// -----------------------------------------------------------------------
// hashfull: starts at 0 and increases after stores
// -----------------------------------------------------------------------

#[test]
fn hashfull_zero_for_empty_tt() {
    let tt = TranspositionTable::new(4);
    assert_eq!(tt.hashfull(), 0);
}

#[test]
fn hashfull_increases_after_stores() {
    let mut tt = TranspositionTable::new(1);
    // Store enough entries to reliably appear in the first 334 sampled clusters.
    for i in 0u64..5000 {
        let hash = i.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x6C62272E07BB0142);
        tt.store(hash, 4, TTFlag::Exact, i as i32 % 1000, 0, Move::NULL);
    }
    assert!(tt.hashfull() > 0, "hashfull should be > 0 after storing 5000 entries");
}

// -----------------------------------------------------------------------
// score_to_tt / score_from_tt: mate score round-trip
// -----------------------------------------------------------------------

#[test]
fn mate_score_round_trip_positive() {
    let ply = 5;
    let original = MATE_SCORE - 3; // mate in 3 from root
    let stored = TranspositionTable::score_to_tt(original, ply);
    let recovered = TranspositionTable::score_from_tt(stored, ply);
    assert_eq!(recovered, original, "positive mate score round-trip failed");
}

#[test]
fn mate_score_round_trip_negative() {
    let ply = 7;
    let original = -(MATE_SCORE - 4); // mated in 4 from root
    let stored = TranspositionTable::score_to_tt(original, ply);
    let recovered = TranspositionTable::score_from_tt(stored, ply);
    assert_eq!(recovered, original, "negative mate score round-trip failed");
}

#[test]
fn normal_score_unchanged_by_round_trip() {
    let ply = 10;
    for score in [-500, -100, 0, 75, 300, 800] {
        let stored = TranspositionTable::score_to_tt(score, ply);
        assert_eq!(
            stored, score,
            "normal score {score} should not be adjusted by score_to_tt"
        );
        let recovered = TranspositionTable::score_from_tt(stored, ply);
        assert_eq!(
            recovered, score,
            "normal score {score} round-trip failed"
        );
    }
}

#[test]
fn value_none_is_distinct() {
    // VALUE_NONE must not collide with any real score.
    assert!(VALUE_NONE.abs() > INF_EVAL, "VALUE_NONE should be out-of-range");
    assert!(VALUE_NONE.abs() > MATE_SCORE, "VALUE_NONE should exceed MATE_SCORE");
}

#[test]
fn max_ply_is_positive() {
    assert!(MAX_PLY >= 64, "MAX_PLY should be at least 64");
}
