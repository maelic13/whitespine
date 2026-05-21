# Changelog

All notable changes to Whitespine are documented in this file.

## [2.0.0] - 2026-05-21

### Added

- Custom Rust board representation based on bitboards, mailbox lookup, fixed
  move lists, incremental make/unmake, and Zobrist hashing.
- Complete legal move generation for standard chess, including castling,
  en passant, promotions, checks, pins, repetitions, the fifty-move rule, and
  insufficient material detection.
- Perft and board correctness test coverage for common reference positions and
  special move edge cases.
- Search benchmark support through the UCI `bench [depth]` command.
- Release benchmark and performance tests for board operations.
- `cargo bench --bench board` board implementation benchmark aligned with the
  `claude` branch workload for cross-branch and cross-engine comparison.
- UCI `Threads` option with Lazy SMP-style persistent-worker search and packed
  shared-transposition-table support up to 1024 threads.
- Epoch-tracked UCI command control with prioritized `quit`, asynchronous
  `stop`/`ponderhit`, serialized idle `isready` handling, and in-order EOF
  shutdown for redirected UCI sessions.
- Threaded root result selection and fixed-depth helper root diversification for
  stronger Lazy SMP behavior during both timed searches and benchmarks.
- Quiet-only legal move generation for qsearch check/promotion expansion.
- Transposition table, pawn cache, correction history, main history, capture
  history, continuation history, killers, and countermoves.
- Advanced search features: PVS, aspiration windows, null-move pruning,
  ProbCut, singular extensions, futility pruning, late move pruning, late move
  reductions, SEE move ordering/pruning, and quiescence search.
- Handcrafted tapered evaluation with material, PeSTO-style piece-square tables,
  pawn structure, passed pawns, mobility, rook activity, outposts, threats,
  king safety, draw scaling, and winning-king push.
- Release profile tuning with fat LTO, one codegen unit, and `panic = "abort"`.

### Changed

- Removed dependency on external chess crates and moved all board/search/eval
  logic into the engine.
- Reworked UCI engine configuration around the custom board and searcher.
- Updated the built-in `bench` command to use the current UCI options,
  including `Threads`.
- Hardened transposition-table resizing so failed large `Hash` allocations keep
  the current table instead of leaving search without TT storage.
- Hardened the shared transposition table with full-key validation to avoid
  cross-thread partial-key collisions.
- Updated qsearch to use quiet-only move generation for early checking and
  promotion moves instead of regenerating the full legal move list.
- Made `ponderhit` reset the active search clock after ponder search converts
  to normal thinking.
- Made helper-thread creation fail gracefully if the OS cannot create every
  requested worker.
- Kept UCI node-limited searches on the main search path to preserve the
  requested node limit when `Threads` is greater than one.
- Updated release version to `2.0.0`.
- Expanded documentation for UCI support, benchmarking, testing, and engine
  internals.

### Tested

- `cargo test --release`
- `cargo bench --bench board`
- Built-in `bench` at 1 and 8 threads.
- Local `chess_tester` 8-thread versus 1-thread validation at 50 ms/move:
  `+29 =9 -2` over 40 games, `+274.6 +/- 141.3` Elo, `100.0%` LOS.
- Local `chess_tester` Stockfish calibration at 20 ms/move:
  `2373 +/- 91` estimated Elo.
- Local head-to-head testing against Basilisk release and Basilisk PEXT builds.

### Notes

- Elo numbers in this changelog are local regression indicators. They are not
  official ratings and should be interpreted with the reported uncertainty and
  test conditions.
