# Whitespine development plan

Whitespine is being rebuilt by its author, by hand, into a strong classical
(hand-crafted evaluation) engine and later an NNUE engine. There are two goals
of equal weight: **learn Rust and chess programming deeply**, and **end with a
classical engine that meets the target defined in "The target" below**, then a
network engine that beats it. Every line of Whitespine code, and every script
around it, is written by the maintainer. Coding agents may explain, review and
edit this document; they do not write code.

This file is the whole roadmap. The **Contents** below is the status board:
tick a box when you judge the item done. Everything under it explains each
item: what it is, how it is usually built, a short illustrative example, the
Rust it exercises, and the end state that counts as done. The examples show a
technique, not a finished solution; they leave out glue, error handling and the
decisions that are yours to make. `EXPERIMENTS.md` holds registrations and
results of every game test, tuning run and fit.

Rust toolchain: **1.98.1**, edition 2024. Chess960 is out of scope by decision.

## Contents

### Phase 0 — Foundations, instruments and the 1.4.0 baseline

- [x] **0.1** Toolchain pin, release profile and lint policy
- [ ] **0.2** Repository hygiene: tracked lock file, line endings, honest README
- [ ] **0.3** Library and binary split; module plan
- [ ] **0.4** Continuous integration on every push
- [ ] **0.5** Game-testing lab: fastchess, books, command templates
- [ ] **0.6** Bug-fix release 1.4.1 on the current code
    - [ ] **0.6.1** UCI regression tests that reproduce the audit's bugs
    - [ ] **0.6.2** Protocol and control fixes
    - [ ] **0.6.3** Clock fixes
    - [ ] **0.6.4** Search correctness fixes
    - [ ] **0.6.5** Gate and release 1.4.1
- [ ] **0.7** Reference ladder and the baseline
    - [ ] **0.7.1** Protocol and clock check of every candidate opponent
    - [ ] **0.7.2** Tier-1 calibration round robin with Whitespine 1.4.0 and 1.4.1
    - [ ] **0.7.3** Record the starting point in EXPERIMENTS.md

### Phase 1 — Board representation

- [ ] **1.1** Primitive types
    - [ ] **1.1.1** Color, PieceType and Piece
    - [ ] **1.1.2** Square, File and Rank
    - [ ] **1.1.3** Bitboard
    - [ ] **1.1.4** Move encoding
    - [ ] **1.1.5** Castling rights
- [ ] **1.2** Attack generation
    - [ ] **1.2.1** Leaper attack tables in `const fn`
    - [ ] **1.2.2** Reference sliding attacks by ray walking
    - [ ] **1.2.3** Magic bitboards
    - [ ] **1.2.4** Geometry tables: between and line
- [ ] **1.3** Position state and FEN
    - [ ] **1.3.1** Board and state structures
    - [ ] **1.3.2** FEN parsing and writing
    - [ ] **1.3.3** Consistency checker and pretty printer
- [ ] **1.4** Zobrist hashing
    - [ ] **1.4.1** Key generation in const context
    - [ ] **1.4.2** Incremental hash and from-scratch verification
    - [ ] **1.4.3** Pawn, non-pawn and material keys
- [ ] **1.5** Move generation
    - [ ] **1.5.1** Allocation-free move list
    - [ ] **1.5.2** Attack queries, checkers and pins
    - [ ] **1.5.3** Pseudo-legal generation by type: noisy, quiet, evasions
    - [ ] **1.5.4** Legality test for pseudo-legal moves
    - [ ] **1.5.5** Validation of moves from tables (`is_pseudo_legal`)
    - [ ] **1.5.6** `gives_check`
- [ ] **1.6** Make and unmake
    - [ ] **1.6.1** State stack, make and unmake
    - [ ] **1.6.2** Null move
- [ ] **1.7** Game rules: repetition, fifty moves, insufficient material
- [ ] **1.8** Perft and the correctness suite
    - [ ] **1.8.1** `perft` and divide
    - [ ] **1.8.2** Reference positions and the EPD suite
    - [ ] **1.8.3** Differential testing against the `chess` crate
    - [ ] **1.8.4** Board speed baseline
- [ ] **1.9** Static exchange evaluation

### Phase 2 — Engine rebuilt on the new board (release 2.0.0)

- [ ] **2.1** UCI protocol layer
    - [ ] **2.1.1** Typed command parsing
    - [ ] **2.1.2** Search limits: every `go` parameter
    - [ ] **2.1.3** Option registry
    - [ ] **2.1.4** Debug commands and command-line `bench`
    - [ ] **2.1.5** fastchess compliance check
- [ ] **2.2** Search thread and stop control
    - [ ] **2.2.1** Worker thread, atomic stop, no lost commands
    - [ ] **2.2.2** Clock and node checks
    - [ ] **2.2.3** Output: info lines and an always-legal `bestmove`
- [ ] **2.3** Scores and the minimal search
    - [ ] **2.3.1** Integer scores and mate distance by ply
    - [ ] **2.3.2** Port the 1.4.0 evaluation with a differential test
    - [ ] **2.3.3** Alpha-beta, iterative deepening and the PV table
    - [ ] **2.3.4** Quiescence search done right
    - [ ] **2.3.5** MVV-LVA ordering
- [ ] **2.4** Time management v1
- [ ] **2.5** Bench command and node fingerprint
- [ ] **2.6** Remove the `chess` crate
- [ ] **2.7** Gate and release 2.0.0
- [ ] **2.8** Fast-control readiness: SPRTs at 3+0.03

### Phase 3 — Search fundamentals

- [ ] **3.1** Transposition table
    - [ ] **3.1.1** Entry, table and indexing
    - [ ] **3.1.2** Probe, store, replacement and mate adjustment
    - [ ] **3.1.3** Search integration and the Hash option
- [ ] **3.2** Staged move picker, killers and quiet history
    - [ ] **3.2.1** Staged picker with SEE-split captures
    - [ ] **3.2.2** Killer moves
    - [ ] **3.2.3** Quiet history with gravity
- [ ] **3.3** Principal variation search and node types
- [ ] **3.4** Search stack and the improving flag
- [ ] **3.5** Null move pruning
- [ ] **3.6** Reverse futility pruning and razoring
- [ ] **3.7** Late move reductions
- [ ] **3.8** Move-loop pruning: late move pruning, futility, SEE
- [ ] **3.9** Quiescence refinements
- [ ] **3.10** Extensions and depth adjustments: check extension, IIR, mate distance pruning
- [ ] **3.11** Aspiration windows
- [ ] **3.12** Tunable parameters and the SPSA pipeline
    - [ ] **3.12.1** Tunables macro and the tune build
    - [ ] **3.12.2** weather-factory setup
    - [ ] **3.12.3** Pilot run that proves the pipeline
- [ ] **3.13** Checkpoint and release 2.1.0

### Phase 4 — The classical evaluation

- [ ] **4.1** Evaluation architecture
    - [ ] **4.1.1** Value scale, piece values and seeds
    - [ ] **4.1.2** Tapered score type and game phase
    - [ ] **4.1.3** Parameters as data and the coefficient trace
    - [ ] **4.1.4** Symmetry and reconstruction tests
    - [ ] **4.1.5** The `eval` breakdown command
- [ ] **4.2** Material and piece-square tables
    - [ ] **4.2.1** Incrementally updated material and piece-square score
    - [ ] **4.2.2** Table shape and seeds
- [ ] **4.3** Evaluation pipeline and caches
    - [ ] **4.3.1** Evaluation order and the lazy exit
    - [ ] **4.3.2** Material hash table
    - [ ] **4.3.3** Pawn hash table
    - [ ] **4.3.4** Attack maps, king ring, mobility area and blockers
- [ ] **4.4** Pawn structure
- [ ] **4.5** Pieces
    - [ ] **4.5.1** Mobility
    - [ ] **4.5.2** Knights and bishops
    - [ ] **4.5.3** Rooks and queens
- [ ] **4.6** King safety
    - [ ] **4.6.1** Pawn shelter and storm
    - [ ] **4.6.2** King danger
    - [ ] **4.6.3** Flank terms
- [ ] **4.7** Threats
- [ ] **4.8** Passed pawns
- [ ] **4.9** Space
- [ ] **4.10** Material imbalance
- [ ] **4.11** Initiative
- [ ] **4.12** Scale factors
- [ ] **4.13** Syzygy tablebases through vendored Fathom
    - [ ] **4.13.1** Vendoring and `build.rs` with `cc`
    - [ ] **4.13.2** FFI layer and UCI options
    - [ ] **4.13.3** Search integration: interior WDL and root DTZ
- [ ] **4.14** Endgames
    - [ ] **4.14.1** Endgame dispatch by material key
    - [ ] **4.14.2** Mating evaluations: KXK, KBNK, KQKR, KNNKP
    - [ ] **4.14.3** KPK bitbase by retrograde analysis
    - [ ] **4.14.4** Drawish and won evaluations: KRKP, KRKB, KRKN, KQKP, KNNK
    - [ ] **4.14.5** Scaling functions
    - [ ] **4.14.6** Measurement against tablebases
- [ ] **4.15** Complete evaluation checkpoint

### Phase 5 — Tuning the classical evaluation

- [ ] **5.1** Training data
    - [ ] **5.1.1** Self-play data generation
    - [ ] **5.1.2** Position filtering and labels
    - [ ] **5.1.3** Splits and the corpus manifest
- [ ] **5.2** Texel tuner
    - [ ] **5.2.1** Data loading and sparse coefficients
    - [ ] **5.2.2** K fitting and the loss
    - [ ] **5.2.3** Gradient optimiser, validation and export
    - [ ] **5.2.4** Nonlinear terms: king danger, initiative, scale factors
- [ ] **5.3** Fitting manifest: free, fixed and excluded parameters
- [ ] **5.4** First full fit and gate
- [ ] **5.5** Pending families after the fit
- [ ] **5.6** SPSA of the nonlinear residue
- [ ] **5.7** Search margin re-fit on the new evaluation
- [ ] **5.8** Candidate terms beyond the core set
- [ ] **5.9** Refit cycles
- [ ] **5.10** Checkpoint and release 2.2.0

### Phase 6 — Advanced search

- [ ] **6.1** Transposition table 2.0: clusters, stored static eval, TT-PV, ageing
- [ ] **6.2** History family
    - [ ] **6.2.1** Capture history
    - [ ] **6.2.2** Continuation histories
    - [ ] **6.2.3** Threat-aware quiet history and pawn history
- [ ] **6.3** Correction histories and the corrected static eval
- [ ] **6.4** LMR 2.0 and history pruning
- [ ] **6.5** Singular extensions, multi-cut and negative extensions
- [ ] **6.6** ProbCut and null move refinements
- [ ] **6.7** Quiescence 2.0
- [ ] **6.8** Root: root moves, MultiPV and `searchmoves`
- [ ] **6.9** Search SPSA over clusters
- [ ] **6.10** Checkpoint and release 2.3.0

### Phase 7 — Clock, pondering, threads and robustness

- [ ] **7.1** Time management v2
- [ ] **7.2** Pondering
- [ ] **7.3** Lazy SMP
    - [ ] **7.3.1** Shared, thread-safe transposition table
    - [ ] **7.3.2** Helper threads and result selection
    - [ ] **7.3.3** Scaling measurement
- [ ] **7.4** Protocol robustness and engine lifecycle
- [ ] **7.5** Checkpoint: four threads and long time control

### Phase 8 — Performance, release engineering and the classical checkpoint

- [ ] **8.1** Profiling workflow
- [ ] **8.2** Throughput work: caches, prefetch, layout
- [ ] **8.3** PEXT slider attacks
- [ ] **8.4** CPU tiers, PGO and the release matrix
- [ ] **8.5** Final classical refit: evaluation, then search margins
- [ ] **8.6** Classical checkpoint and release 3.0.0

### Phase 9 — NNUE

- [ ] **9.1** The network contract
- [ ] **9.2** Board events for the accumulator
- [ ] **9.3** Data generation at scale
- [ ] **9.4** Training pipeline
- [ ] **9.5** Scalar inference and conformance
- [ ] **9.6** Incremental accumulators
- [ ] **9.7** SIMD inference
- [ ] **9.8** Search re-fit for the network
- [ ] **9.9** Network ladder and data refresh
- [ ] **9.10** NNUE release 4.0.0

---

## The target

The classical Whitespine (release 3.0.0, end of Phase 8) is defined by what it
can do, how it was built, and how its strength was established. Phases 0–8
close the gap between 1.4.0 and this definition; Phase 9 builds on it.

**What the engine has.**

- **A board of its own**: bitboards, constant-time slider attacks, make and
  unmake with a compact undo record, precomputed pins and checkers, static
  exchange evaluation, and hash keys for the position, the pawns, the non-pawn
  pieces and the material. Correctness proven by perft and by differential
  testing against an independent implementation.
- **A modern alpha-beta search**: principal variation search with node types,
  a clustered transposition table with ageing, staged move ordering fed by
  quiet, capture, continuation and pawn-structure histories, correction
  histories that fix the static evaluation from search results, null move
  pruning with verification, ProbCut, reverse futility, razoring, late move
  reductions in fractional units, late move and futility and SEE pruning,
  singular extensions with multi-cut and negative extensions, aspiration
  windows, MultiPV and `searchmoves`.
- **A complete hand-crafted evaluation**: tapered integer scores for material,
  piece-square tables, material imbalance, pawn structure, mobility, piece
  placement terms, king safety with a nonlinear danger model, threats, passed
  pawns, space, initiative and endgame scale factors; an endgame library with
  specialised evaluations, scaling functions and a KPK bitbase; every term
  traced so it can be fitted.
- **Every constant fitted, none guessed**: evaluation parameters fitted to
  results of the engine's own games; nonlinear evaluation constants, search
  margins and clock factors tuned by games; every change accepted by a
  registered game test.
- **Endgame tablebases**: Syzygy probing at the root (distance to zeroing) and
  inside the search (win, draw, loss).
- **A clock that never forfeits**: soft and hard limits, scaled by best-move
  stability, effort and score trend; pondering; zero time losses across fast,
  slow and repeating controls.
- **Threads**: lazy SMP over a shared lock-free table, with a measured scaling
  curve.
- **A complete and robust protocol**: every `go` parameter, every option
  validated, no lost command, no illegal or null move, no crash under a
  scripted stress test or a long tournament.
- **Instruments**: perft suites, a deterministic bench fingerprint, an SPRT
  lab, an SPSA pipeline, a Texel tuner, a data generator, continuous
  integration on three operating systems, and a release matrix of per-CPU-tier
  profile-guided builds that all print one fingerprint.

**How it was built.** By hand, feature by feature, each one measured: a
behaviour-neutral change reproduces the bench fingerprint exactly; every other
change is accepted or rejected by a registered game test whose prediction was
written before the first game. Ideas come from the chess programming
literature; the code and the constants are Whitespine's own.

**How strong it is.** Strength is measured, never asserted. Every checkpoint
plays a gauntlet against a reference ladder of opponents at known relative
strength (0.7), so the rating history from 1.4.0 to 3.0.0 is a set of
measurements against the same anchor. The ambition for 3.0.0 is the strength
band of mature classical engines, which public rating lists place at roughly
3,000 Elo and above at their standard controls; that band is represented in
the ladder by its top tier and is reached when Whitespine's measured rating
sits among that tier's engines. The checkpoint at 8.6 writes down where every
accepted Elo came from, phase by phase, so the number has an explanation.

**The network Whitespine** (release 4.0.0, end of Phase 9) replaces the
hand-crafted evaluation with a neural network of its own design, trained only
on data Whitespine itself generated, and beats 3.0.0 at short and long
controls and at four threads.

## Where we start

### Whitespine 1.4.0 against the target

Whitespine 1.4.0 (`bf06899` plus logo commits, 2026-09-14) is 1,158 lines in
eight files. Everything chess-related is delegated to the `chess` crate 3.2.0:
board, move generation, legality and game history. The comparison below is what
Phases 0–8 of this plan close.

| Area | Whitespine 1.4.0 | Target end state (Phase 8) |
|---|---|---|
| Board | `chess` crate `Game`, rebuilt by replaying the whole game on every access | Own bitboards, magic and PEXT sliders, make/unmake with a compact undo record, pins and checkers, SEE, pawn/non-pawn/minor keys, threat maps |
| Move generation | Legal moves collected into `Vec`, full board copy per move for check detection | Allocation-free lists, noisy/quiet/evasion generation, pseudo-legal validation for table moves, `gives_check` |
| Search | Fail-hard alpha-beta, iterative deepening, quiescence with captures and every check | PVS with node types, TT cutoffs, NMP, ProbCut, RFP, razoring, LMR in fractional units, LMP, futility and SEE pruning, singular/multi-cut/negative extensions, aspiration windows, MultiPV |
| Transposition table | None | 3-entry 32-byte clusters with stored static eval, TT-PV flag and 5-bit age |
| Move ordering | MVV-LVA plus a "gives check" bonus, recomputed by copying the board | Staged picker; quiet, capture, continuation and pawn histories with gravity; SEE-split captures |
| Evaluation corrections | None | Pawn, non-pawn and continuation correction histories, rule-50 damping |
| Evaluation | Untapered centre and king-distance bonuses in `f64` | Tapered, integer, traced families: pawns, passers, mobility, pieces, king safety, threats, space, imbalance, scaling, endgame recognisers, KPK |
| Tuning | Hand-picked constants | Texel fits of the whole linear surface, SPSA of search and nonlinear coordinates, registered SPRTs |
| Tablebases | None | Syzygy through vendored Fathom with root DTZ and interior WDL |
| Clock | 5% of the clock, or 10% plus the increment; can go negative | Soft and hard bounds, stability and node-fraction scaling, forfeit margin |
| Threads | `Threads` advertised, clamped to 1 | Lazy SMP with a shared table |
| Protocol | Subset of `go`; commands received during a search are lost | Full `go` limits, pondering, MultiPV, robust lifecycle |
| Instruments | One unit test | Perft suites, bench fingerprint, SPRT harness, SPSA, Texel tuner, datagen, CI matrix, PGO tier builds |

**Conclusion.** Almost nothing in 1.4.0 is a base to build on; what carries
over is the experience of having written an engine, the UCI shell shape and the
release workflow. The rewrite is gradual (agreed): the new board grows in the
same crate, is proven against the `chess` crate by differential tests, and the
crate is removed only when the engine runs entirely on the new code (2.6).
1.4.0 stays tagged; the bug-fixed 1.4.1 (0.6) is the rewrite's first opponent.

### Audit of 1.4.0

Every finding names where it is removed: bugs that are local to today's code
are fixed first in the 1.4.1 bug-fix release (0.6) with a UCI regression test,
then re-implemented correctly by the rewrite, where the same test proves they
stay fixed (2.7); structural defects are removed by the rewrite alone.
"Measured" means observed on the
1.4.0 release build on 2026-09-14 with short single-threaded UCI sessions on a
lightly loaded host (rough numbers, not a speed study); the rest are read from
the source.

| ID | Severity | Finding | Evidence | Fixed by |
|---|---|---|---|---|
| W-01 | Critical | `Game::current_position()` replays every move from the start position on each call, and `Game::can_declare_draw()` replays the game and generates legal moves for every earlier position. Both run several times per node, and each child also clones the game's move history (`engine.rs:186`, `engine.rs:271`). Node cost grows with game length. | Measured: depth 5 from the start position ~239k NPS; the same search after 56 plies ~26k NPS, nine times slower | Rewrite: 1.6, 1.7, 2.3.3 |
| W-02 | Critical | When a draw *can be claimed* at the root, the engine prints `bestmove 0000` instead of a move (`engine.rs:82`). A GUI that does not claim the draw itself receives a null move. | Measured: after `g1f3 g8f6 f3g1 f6g8` twice, `go depth 3` answers `bestmove 0000` | 1.4.1: 0.6.2; rewrite: 2.2.3 |
| W-03 | High | Quiescence stands pat while in check and considers only captures and checks, so check evasions by quiet moves are never searched (`engine.rs:231-234`, `engine.rs:296`). Mates and forced losses are misjudged at the horizon. | Source | 1.4.1: 0.6.4; rewrite: 2.3.4 |
| W-04 | High | Quiescence generates every checking move at every quiescence ply with no limit (`engine.rs:296-318`); check sequences are bounded only by threefold repetition. | Source | 1.4.1: 0.6.4; rewrite: 2.3.4 |
| W-05 | High | Mate scores use *remaining depth*, not distance from the root (`heuristic.rs:61-63`); quiescence scales mate scores by 0.95 (`engine.rs:225`); mates are reported as `score cp`. The same mate changes score between iterations. | Measured: mate in one reported as `cp 12002` at depth 3 and `cp 12003` at depth 4 | 1.4.1: 0.6.4; rewrite: 2.3.1 |
| W-06 | High | Iterative deepening carries nothing between iterations (no table, previous best move not tried first), so each iteration repeats the whole tree. | Measured: 17,167 → 126,882 → 1,013,546 nodes at depths 4 → 5 → 6, a branching factor near 8 | Rewrite: 3.1, 3.2, 3.3 |
| W-07 | High | The search polls the command channel at every node and silently discards any command that is not `stop` or `quit` (`engine.rs:68-76`). The poll plus a clock read also run at every node, quiescence included. | Measured: `go infinite`, `go depth 1`, `stop` produce one `bestmove` | 1.4.1: 0.6.2; rewrite: 2.2.1, 2.2.2 |
| W-08 | Medium | If stopped before depth 1 finishes, the move played is chosen by the clock's nanoseconds from the legal list (`engine.rs:92`). | Measured: `bestmove a2a4` after an immediate stop | 1.4.1: 0.6.2; rewrite: 2.2.3 |
| W-09 | High | Clock: without increment the budget is `0.05 × (time − overhead)`, negative when time is below the overhead; with increment it is `0.1 × time + inc`, capped only at `time − overhead`; `movestogo` is ignored; the overhead is not applied to `movetime`; timing starts when the engine thread dequeues the command, not at `go` (`engine.rs:321-356`). | Measured: `go wtime 5 btime 5` answers `g2g3` immediately without a search line | 1.4.1: 0.6.3; rewrite: 2.4, 7.1 |
| W-10 | Medium | `go nodes`, `mate`, `movestogo`, `searchmoves` and `ponder` are ignored; `go nodes N` searches until stopped; `go` without arguments means depth 2 (`search_options.rs:104-143`, `:114`). | Measured: no `bestmove` within 3 s for `go nodes 1000` | 1.4.1: 0.6.2; rewrite: 2.1.2 |
| W-11 | Medium | Scores and depth are `f64`; the king's value is `f64::INFINITY`, which `as i32` saturates to `i32::MAX`, so every capture by the king sorts last in MVV-LVA (`piece_value.rs:20`, `engine.rs:371`). | Source | 1.4.1: 0.6.4; rewrite: 1.1, 2.3.1, 2.3.5 |
| W-12 | Medium | Move ordering copies the board for every move to find checks (`engine.rs:381`); the quiescence generator orders all legal moves (another copy each) and copies again (`engine.rs:303`). | Source | Rewrite: 1.5.6, 3.2 |
| W-13 | Medium | Fail-hard returns (`engine.rs:203`, `:234`, `:286`) waste bound information; the PV is built with `Vec::insert(0, …)`, allocating at every node (`engine.rs:200`). | Source | Rewrite: 2.3.3 |
| W-14 | Medium | Evaluation is untapered; the king-distance term `14/d × w − w` reaches 13w (104 cp for a knight or queen next to the enemy king, 65 cp for a pawn) and rewards pawns for approaching the enemy king (`heuristic.rs:328`); there is no pawn structure, mobility, king safety, passed pawn or bishop-pair knowledge; no constant is fitted. | Source | Rewrite: Phase 4 (seeded), Phase 5 (fitted) |
| W-15 | Low | Protocol hygiene: `Invalid setoption command.` is printed without `info string` (`search_options.rs:153`); a banner line precedes `uci` (`main.rs:18`); a non-UTF-8 input line panics (`uci_protocol.rs:26`); no `Hash` option; an illegal move in `position` silently keeps the previous position. | Measured for `setoption` and the illegal move | 1.4.1: 0.6.2; rewrite: 2.1 |
| W-16 | Low | Delta pruning has two identical branches (clippy `if_same_then_else`, `engine.rs:255`) and ignores promotions. | `cargo clippy` | 1.4.1: 0.6.4; rewrite: 3.9 |
| W-17 | Medium | Engineering: `Cargo.lock` is ignored for a binary crate, so builds are not reproducible; no toolchain pin; CI runs only on published releases; one unit test; 18 clippy warnings; no perft or bench; `pub fn default()` in place of the `Default` trait. | `.gitignore`, `cargo clippy`, `cargo test` | 0.1–0.4, 2.5 |

---

## Working rules

These are the maintainer's rules for this plan.

1. **Done means the maintainer says so.** Each item lists an end state and a
   suggested check; the judgement is yours. A tick is never a claim about
   strength unless a game test says so.
2. **Know which layer a number lives in.** Perft counts prove rules; bench
   node counts prove behaviour identity; NPS proves speed; fit loss proves a
   fit converged; tactical-suite and node counts screen search changes. Only
   games measure strength, and none of the other layers converts to Elo.
3. **Write the prediction before the games.** Register every SPRT, gauntlet,
   SPSA and fit in `EXPERIMENTS.md` before it starts: baseline, candidate,
   bounds, time control, book, expected result. Afterwards add the result and
   what the prediction got wrong. A rejected or neutral result is a result.
4. **Measure what you think you measure.** Build the release binary from
   committed source, name it with version and short SHA, record its SHA-256
   and its bench signature, and copy it to the lab before the match. A stale
   binary is the most common way to measure nothing. Check a harness option is
   live by setting an absurd value and watching the numbers move.
5. **One heavy job at a time on an idle host.** Other projects share this
   machine. Before a match, check that nothing else is playing, building or
   profiling; never build while a match runs. A match that ran on a loaded
   host measures the load, not the engines.
6. **Behaviour-neutral means an identical bench node count.** Refactors,
   renames and speed work must reproduce the node count exactly (2.5). A
   changed count is a behaviour change and needs a game test.
7. **Test in debug and in release.** Debug builds catch overflow and
   `debug_assert!`; release builds catch timing and optimisation-dependent
   bugs. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
   stay clean.
8. **The toolchain moves only between gates.** Never bump Rust between building
   a baseline and its candidate; a compiler change is its own measured event.
9. **Sources teach, you write.** Read the Chess Programming Wiki, papers and
   open-source engines for mechanisms and pitfalls, then close them and write
   your own code from the description in this plan. A constant taken from
   anywhere else is a seed on someone else's scale, to be fitted, not a result.
10. **Engine commits and document commits are separate.** Tag every SPRT
    baseline you might need again (for example `base/2.1.0` or `exp/ws-017`) so
    old binaries can be rebuilt without branches.
11. **Features early, clusters late.** Phases 2–5 gate one feature at a time:
    each is worth tens of Elo on its own and the gains are easy to see. From
    Phase 6 on, mechanisms feed each other (histories drive LMR and pruning,
    corrections drive margins); implement a dependency-complete group, tune it
    together, and gate the group.

### Game-test policy

| Purpose | Bounds (normalized Elo) | Notes |
|---|---|---|
| Rewrite gate, first TT, replacing the evaluation, first full Texel fit, other large expected gains | `[0, 10]` | Resolves in hundreds to a few thousand games |
| Ordinary feature in Phases 3–5 | `[0, 5]` | |
| Refinement in Phases 5–8 | `[0, 3]` | Tens of thousands of games; budget overnight; confirm big clusters at 10+0.1 |
| Bug-fix release (1.4.1) | `[-5, 0]` | Accepts "not worse"; correctness is the reason for the release |
| Simplification or removal | `[-1.75, 0.25]` | Accepts "not worse" within a small loss |
| Repair of unknown sign | `[-5, 5]` | |

**Time control.** 8+0.08 until 2.8, because the old engine and the first
rewrite are not yet proven at fast controls. From 2.8 on, **3+0.03** for SPRTs
and SPSA, with long-control confirmations at **10+0.1**. A fast control plays
several times more games per hour, which is what decides whether a
few-Elo change can be resolved overnight; a long-control confirmation guards
against changes that only help when the search is shallow. If a later change
produces time forfeits, the default returns to 8+0.08 until they are fixed.

All SPRTs: fastchess `model=normalized`, `alpha=0.05 beta=0.05`, paired
openings (`-repeat`), one thread per engine, `-use-affinity`, concurrency 14,
`Hash=16`, UHO book, **no adjudication**. Multi-thread tests drop affinity.
Never change bounds, book, time control or cap after the first game.

---

## Phase 0 — Foundations, instruments and the 1.4.0 baseline

**Why first.** Everything later is measured. Before the first line of the new
board exists you want reproducible builds, a CI that runs tests on every push,
a place where games are played the same way every time, and a measured
starting strength. None of this changes how 1.4.0 plays.

### 0.1 Toolchain pin, release profile and lint policy

**What.** Pin the compiler so a baseline and a candidate are always built by
the same Rust; set the release profile an engine needs; decide which lints
fail the build.

**How it is usually done.** A `rust-toolchain.toml` at the repository root
makes `cargo` download and use exactly that toolchain in this directory,
including on CI. Engines build with fat LTO and one codegen unit (whole-program
inlining of the hot paths) and `panic = "abort"` (smaller, slightly faster; a
panic kills the process, which is what you want from an engine anyway). Lints
live in `Cargo.toml` so every build and CI job sees the same policy.

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.98.1"
components = ["rustfmt", "clippy", "llvm-tools-preview"]  # llvm-tools for PGO in 8.4
profile = "minimal"
```

```toml
# Cargo.toml
[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"

[profile.profiling]          # release speed with symbols, for 8.1
inherits = "release"
debug = true

[lints.clippy]
all = { level = "warn", priority = -1 }
```

**Rust you will practise.** Cargo profiles and inheritance, the `[lints]`
table, what `panic = "abort"` changes (no unwinding, `catch_unwind` stops
working).

**End state.** `cargo build --release` in a fresh clone uses 1.98.1; the
release profile is set; `cargo clippy --all-targets -- -D warnings` passes
after fixing today's 18 warnings.

**Check.** `rustup show active-toolchain` inside the repository names 1.98.1;
clippy exits 0.

### 0.2 Repository hygiene: tracked lock file, line endings, honest README

**What.** Make the repository reproduce the binary and say only true things.

**How.** A binary crate commits `Cargo.lock`; today `.gitignore` ignores it
(W-17). Add a `.gitattributes` so line endings stop depending on each
machine's `core.autocrlf` (for example `* text=auto` and `*.rs text eol=crlf` if
you keep CRLF, or LF everywhere). Update the README feature list whenever a
phase lands, never ahead of it. Keep `uci_specification.txt`: you will read it
more than once in Phase 2.

**End state.** `Cargo.lock` tracked; `.gitattributes` present; README
describes 1.4.0 as it is.

**Check.** `git ls-files Cargo.lock` prints the file; `git status` is clean
after a fresh clone and build.

### 0.3 Library and binary split; module plan

**What.** Turn the crate into a library plus a thin binary, so that tests,
benchmarks and tools (datagen in 5.1, the tuner in 5.2) can use the engine
code, and lay out the module tree the plan fills.

**How.** A package with both `src/lib.rs` and `src/main.rs` builds a library
named after the package and a binary that can `use whitespine::…`. Integration
tests in `tests/` see only the library's public items, which forces you to
think about visibility early.

```rust
// src/main.rs — the binary stays a few lines long
fn main() {
    whitespine::uci::run(std::env::args().skip(1));
}
```

A module plan that later phases fill (create folders as you reach them):

```text
src/
  lib.rs            pub mod board; pub mod search; pub mod eval; pub mod uci; ...
  board/            types.rs, bitboard.rs, attacks.rs, magic.rs, zobrist.rs,
                    movegen.rs, position.rs, see.rs, perft.rs
  search/           mod.rs, tt.rs, movepick.rs, history.rs, time.rs, params.rs
  eval/             mod.rs, score.rs, params.rs, trace.rs, pawns.rs, king.rs, ...
  uci/              mod.rs, command.rs, options.rs, bench.rs
tools/              separate workspace crates: datagen (5.1), texel (5.2)
tests/              integration tests: perft, FEN, UCI sessions
```

**Rust you will practise.** Crates versus modules, `pub`, `pub(crate)` and
`pub(super)`, `mod.rs` versus `name.rs` layout, Cargo workspaces (you already
use one in Advent of Code).

**End state.** 1.4.0's behaviour unchanged, now living in a library with a
two-line `main`. The current modules move under the new layout only as far as
is cheap; the rewrite replaces them anyway.

**Check.** `cargo test` passes; a UCI session gives the same `bestmove` for
`position startpos` / `go depth 5` as before.

### 0.4 Continuous integration on every push

**What.** A GitHub Actions workflow that runs on every push and pull request,
separate from the existing release workflow.

**How.** One job per operating system (Windows, Linux, macOS), each running
format check, clippy, and tests in debug and in release. Later phases add
steps: shallow perft (1.8), bench signature (2.5), and an assertion that every
release asset prints the same bench signature (8.4).

```yaml
# .github/workflows/ci.yml (skeleton)
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        os: [windows-latest, ubuntu-latest, macos-latest]
        profile: [dev, release]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v6
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --profile ${{ matrix.profile }}
```

**End state.** A push that breaks formatting, clippy or a test is red within
minutes on all three systems.

**Check.** Push a deliberate clippy warning on a throwaway branch and watch
the job fail; then delete the branch.

### 0.5 Game-testing lab: fastchess, books, command templates

**What.** A fixed place, outside the repository, where every game test runs
the same way.

**How.** fastchess 1.8.0 (a release download) plays UCI engines, runs SPRTs,
writes PGN and checks protocol compliance. It does **not** speak xboard, which
is why xboard-only engines are excluded from the ladder (0.7). Another
tournament manager with SPRT support may replace it later; until then
fastchess runs every test. Suggested layout:

```text
D:/chess/whitespine-lab/
  bin/        whitespine-2.0.0-dev-<sha>.exe, whitespine-1.4.0.exe, opponents
  books/      UHO_Lichess_4852_v1.epd, SuperGM_4mvs.pgn, IM_4mvs.pgn
  results/    one folder per experiment ID: PGN, log, command, binary hashes
  tuner/      weather-factory working folder (3.12)
```

The books are public downloads: the UHO books are published by their author
with the Lichess-derived openings, and the balanced 4-move books circulate
with the common testing frameworks. Record each book's SHA-256 in the lab once
and never edit a book file.

UHO is an unbalanced-opening book: every opening gives one side an edge, and
pairing (each opening played with both colours) turns that into sensitivity.
Use it for self-play SPRTs. For gauntlets against much weaker or older engines
use a balanced book (`SuperGM_4mvs.pgn` or `IM_4mvs.pgn`), where UHO's edges
would dominate the result.

SPRT template (PowerShell; restated wherever a step needs it). The time
control is 8+0.08 until 2.8 makes the engine ready for 3+0.03, which is then
the default:

```powershell
& D:\chess\whitespine-lab\bin\fastchess.exe `
  -engine cmd=D:\chess\whitespine-lab\bin\ws-candidate.exe name=candidate `
  -engine cmd=D:\chess\whitespine-lab\bin\ws-baseline.exe name=baseline `
  -each tc=8+0.08 option.Hash=16 option.Threads=1 `
  -openings file=D:\chess\whitespine-lab\books\UHO_Lichess_4852_v1.epd format=epd order=random `
  -repeat -rounds 100000 -concurrency 14 -use-affinity `
  -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 model=normalized `
  -pgnout file=D:\chess\whitespine-lab\results\WS-000\games.pgn `
  -log file=D:\chess\whitespine-lab\results\WS-000\fastchess.log level=warn
```

Gauntlet template (one engine against a list, balanced book, longer clock for
old engines):

```powershell
& D:\chess\whitespine-lab\bin\fastchess.exe -tournament gauntlet `
  -engine cmd=D:\chess\whitespine-lab\bin\ws-candidate.exe name=Whitespine `
  -engine cmd=D:\chess\whitespine-lab\bin\opponent-a.exe name=opponent-a `
  -engine cmd=D:\chess\whitespine-lab\bin\opponent-b.exe name=opponent-b `
  -each tc=10+0.1 `
  -openings file=D:\chess\whitespine-lab\books\SuperGM_4mvs.pgn format=pgn order=random `
  -repeat -rounds 100 -concurrency 14 -use-affinity `
  -pgnout file=D:\chess\whitespine-lab\results\WS-000\gauntlet.pgn
```

`option.Hash` is set per engine only for engines that expose it; fastchess
warns and plays on when an option does not exist, so check the log for that
warning before trusting a result. Read `fastchess -help` once completely.

**End state.** The folder exists; both templates run a 10-game smoke match to
completion; you know where PGN and logs land.

**Check.** The smoke PGN contains 10 games with results, and the log contains
no "option not found" warnings.

### 0.6 Bug-fix release 1.4.1 on the current code

**What.** Fix the audit's correctness and protocol bugs in the code that exists
today, and release 1.4.1. The rewrite (Phases 1–2) replaces this code, so why
fix it? Three reasons: 1.4.0 is published and its users get null moves and
time losses; the first SPRT of the rewrite (2.7) should be measured against an
engine that plays by the rules, not one that forfeits; and the regression
tests you write here (0.6.1) are UCI sessions that run against any binary, so
they carry over unchanged and prove the rewrite does not bring the bugs back.
It is also a gentle start: small, contained fixes in code you already know.

**Which bugs.** The local ones: W-02, W-03, W-04, W-05, W-07, W-08, W-09,
W-10, W-11, W-15, W-16. The structural ones (W-01 game replay, W-06 nothing
carried between iterations, W-12 board copies, W-13 fail-hard and PV
allocation, W-14 evaluation) are what the rewrite is for; W-17 is closed by
0.1–0.4.

**Traceability.** Every audit finding ends in one of three places: a 0.6 fix
with a regression test, a named step of the rewrite, or 0.1–0.4. The audit
table's "Fixed by" column lists them; 2.7 re-runs every 0.6.1 test against the
rewritten engine.

#### 0.6.1 UCI regression tests that reproduce the audit's bugs

**What.** Integration tests that start the engine binary, talk to it over
standard input and output, and check its answers. Write each test first, see it
fail on 1.4.0, then fix the bug.

**How.** `std::process::Command` with piped `stdin` and `stdout`, a reader
thread or a line loop with a timeout, and assertions on the lines received.
Cargo builds the binary for integration tests and exposes its path in the
`CARGO_BIN_EXE_<name>` environment variable.

```rust
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn claimable_draw_at_root_still_returns_a_move() {
    let mut engine = Command::new(env!("CARGO_BIN_EXE_whitespine"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("engine starts");
    let mut input = engine.stdin.take().expect("stdin is piped");
    writeln!(input, "position startpos moves g1f3 g8f6 f3g1 f6g8 g1f3 g8f6 f3g1 f6g8").unwrap();
    writeln!(input, "go depth 3").unwrap();
    let output = BufReader::new(engine.stdout.take().expect("stdout is piped"));
    let bestmove = output
        .lines()
        .map(|line| line.expect("utf-8 output"))
        .find(|line| line.starts_with("bestmove"))
        .expect("a bestmove line");
    assert_ne!(bestmove, "bestmove 0000");
    writeln!(input, "quit").unwrap();
}
```

One test per measured symptom: the claimable draw (W-02); `go infinite`, `go
depth 1`, `stop` producing two `bestmove` lines (W-07); a mate in one reported
as `score mate 1` at every depth (W-05); `go wtime 5 btime 5` producing an
`info` line before `bestmove` (W-09); `go nodes 1000` finishing on its own
(W-10); an invalid `setoption` answered with `info string` (W-15); an
immediate `stop` returning the first ordered move rather than a random one
(W-08). Search bugs (W-03, W-04) get position tests: a mate that needs a quiet
evasion at the horizon, and a check-rich position whose quiescence node count
stays bounded.

**Rust you will practise.** Integration tests in `tests/`,
`std::process::Command`, pipes, `BufRead::lines`, timeouts with a helper
thread, `env!`.

**End state.** A `tests/uci_regressions.rs` whose tests fail on 1.4.0 for the
right reason.

#### 0.6.2 Protocol and control fixes

**What.** W-02: remove the "claimable draw at the root" early return; print
`bestmove 0000` only without legal moves. W-07: replace the command polling in
the search with an `Arc<AtomicBool>` stop flag set by the UCI thread, so no
command is consumed by the search. W-08: before depth 1 completes, fall back to
the first move of the ordered list, not a random one. W-10: parse `nodes`,
`mate`, `movestogo`, `searchmoves` and `ponder`; honour `nodes`; `go` without
parameters means infinite. W-15: every diagnostic as `info string`, no start-up
banner, invalid UTF-8 input ignored instead of panicking, an illegal move in
`position` reported.

**End state.** The corresponding 0.6.1 tests pass.

#### 0.6.3 Clock fixes

**What.** W-09: a budget that can never be negative (`saturating_sub` on the
overhead, a minimum think time), `movestogo` used when given, the overhead
applied to `movetime`, the clock started when `go` is parsed and passed to the
search, and a cap so no single move spends more than a fraction of the
remaining time.

**End state.** The clock test passes; 200 self-play games at 1+0.01 end with
zero time losses.

#### 0.6.4 Search correctness fixes

**What.** W-03: in quiescence, no stand-pat while in check, and all legal
evasions searched. W-04: quiescence generates captures only (checks only as
evasions). W-05: mate scores by distance from the root and `score mate N`
output, no 0.95 scaling. W-11: a finite king value in move ordering. W-16:
the duplicated delta-pruning branches merged, promotions exempt.

**End state.** The search tests pass; `bench`-like fixed-depth runs on a few
positions still find the same or better moves (spot-checked by hand).

#### 0.6.5 Gate and release 1.4.1

**How.** Register an SPRT of 1.4.1 against 1.4.0 with bounds `[-5, 0]`
(accepts "not worse": bug fixes are released for correctness, and a gain is
welcome but not required), 8+0.08, UHO book, no `Hash` option on either side.
Record time losses for both engines: 1.4.0's are part of the evidence.

```powershell
& D:\chess\whitespine-lab\bin\fastchess.exe `
  -engine cmd=D:\chess\whitespine-lab\bin\whitespine-1.4.1.exe name=ws-1.4.1 `
  -engine cmd=D:\chess\whitespine-lab\bin\whitespine-1.4.0.exe name=ws-1.4.0 `
  -each tc=8+0.08 option.Threads=1 `
  -openings file=D:\chess\whitespine-lab\books\UHO_Lichess_4852_v1.epd format=epd order=random `
  -repeat -rounds 100000 -concurrency 14 -use-affinity `
  -sprt elo0=-5 elo1=0 alpha=0.05 beta=0.05 model=normalized `
  -pgnout file=D:\chess\whitespine-lab\results\WS-001\games.pgn
```

**End state.** SPRT recorded; 1.4.1 tagged and released with a changelog that
names the fixed findings.

### 0.7 Reference ladder and the baseline

**What.** A set of opponents at known relative strength, so every checkpoint
can say how strong Whitespine is and a result against itself (SPRT) is never
the only view.

**How it is usually done.** Collect UCI engines spanning the strength range
from below 1.4.0 to the target band, play round robins or gauntlets, and
compute ratings relative to **one** anchor (Ordo or BayesElo on the PGN, or
fastchess's own report). Anchor exactly one engine: several anchors that
disagree smear their disagreement over everything else, and at fast controls
engines anchored to a public list can disagree by hundreds of Elo. Public
rating-list numbers are measured at other controls on other hardware and do
not transfer; use them only to guess which tier an engine belongs to before it
is measured.

**The tiers.** A tier is a band of relative strength with a job in this plan.
Which engines fill each tier is a measurement, recorded in `EXPERIMENTS.md` by
0.7.1 and revised at every checkpoint; the plan itself names no opponent.

| Tier | Job | What belongs there |
|---|---|---|
| 1 | Phase 0 baseline, gate of Phase 2 | Engines around and below 1.4.0's strength: simple hobby engines, and Whitespine 1.4.0 and 1.4.1 themselves |
| 2 | Checkpoint 3.13 | Engines a few hundred Elo above 1.4.0: early-generation hobby engines with a transposition table and basic pruning |
| 3 | Checkpoints 4.15, 5.10, 6.10 | Mature amateur engines with a full classical evaluation |
| 4 | Checkpoints 6.10, 7.5 | Commercial and open-source engines of the late 2000s and early 2010s |
| 5 | Classical target, 8.6 | The strongest hand-crafted-evaluation engines: the band described in "The target" |

Every tier needs at least three engines so a single odd opponent cannot
dominate a gauntlet. Engines that speak only xboard are excluded, since
fastchess cannot run them. An engine with a strength-limiting option can fill a
gap between tiers as an opponent, but its setting is calibrated for another
control and is never the anchor.

#### 0.7.1 Protocol and clock check of every candidate opponent

**How.** Run each engine through `fastchess --compliance <path>`, then a
20-game match against Whitespine 1.4.0 at 10+0.1. Old engines often lose on
time at fast controls or mishandle `ucinewgame`; one that forfeits more than 1%
of games at the chosen control or crashes is dropped or moved to a slower
control.

**End state.** The ladder as a table in `EXPERIMENTS.md`: every opponent with
its tier guess, path, `id name`, SHA-256, and the result of its compliance
and clock check.

#### 0.7.2 Tier-1 calibration round robin with Whitespine 1.4.0 and 1.4.1

**How.** A round robin of Tier 1 with Whitespine 1.4.0 and 1.4.1, balanced
book, 10+0.1, at least 100 games per pair, no adjudication. Choose one Tier-1
engine as the anchor at 0 and compute ratings. The anchor stays the same for
the life of the plan; when Whitespine outgrows Tier 1, later gauntlets are
chained to it through engines that played in both.

**End state.** A relative rating list with error bars; the places of 1.4.0 and
1.4.1 in it.

#### 0.7.3 Record the starting point in EXPERIMENTS.md

**End state.** An observation entry: the round robin's command, engine hashes,
PGN path and ratings, plus the measured facts from the audit (NPS at the start
position and after 56 plies, depth reached at 10+0.1). Phase 2's gate and every
checkpoint compare against this entry.

---

## Phase 1 — Board representation

**Why this order.** Search and evaluation are only as fast and correct as the
board under them. The board is also the best Rust school in the project: small
`Copy` types, operator traits, `const` evaluation, iterators, tests. The new
board grows **next to** the old engine; nothing in 1.4.0 uses it until Phase 2,
and the `chess` crate stays as a test oracle until 2.6.

**End state of the phase.** A library module that parses and writes FEN,
generates noisy, quiet and evasion moves, validates moves, makes and unmakes
moves and null moves, detects repetitions and draws, computes SEE, and passes
the perft suite and a differential test against the `chess` crate over millions
of positions — with its perft speed recorded.

### 1.1 Primitive types

**What.** The vocabulary every other module speaks: colours, pieces, squares,
sets of squares, moves and castling rights. Get these right and most later
code reads like chess.

**Rust you will practise.** Newtypes, `#[repr(u8)]` enums, `#[derive]` on
`Copy` types, `const fn`, operator traits (`Not`, `BitAnd`, `Shl`), `Iterator`,
`Display` and `FromStr`, unit tests in `#[cfg(test)] mod tests`.

#### 1.1.1 Color, PieceType and Piece

**How.** Small enums with explicit discriminants, so they can index arrays.
Keep the conversion to `usize` in one method instead of `as usize` scattered
through the code.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const fn index(self) -> usize {
        self as usize
    }
}

impl std::ops::Not for Color {
    type Output = Color;
    fn not(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
```

`PieceType` has six variants; `Piece` combines colour and type (twelve values).
Decide whether `Piece` is its own enum or a packed `u8` (`color << 3 | type`).
Either works; an enum makes `Option<Piece>` one byte through the niche
optimisation.

**End state.** Types with `index()`, `ALL` constants for iteration, and
conversions to and from FEN characters, with tests.

#### 1.1.2 Square, File and Rank

**How.** A square is a `u8` from 0 (a1) to 63 (h8). Wrap it in a newtype so a
square cannot be confused with a count or an index into something else.

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Square(u8);

impl Square {
    pub const fn new(file: u8, rank: u8) -> Square {
        Square(rank * 8 + file)
    }
    pub const fn file(self) -> u8 {
        self.0 & 7
    }
    pub const fn rank(self) -> u8 {
        self.0 >> 3
    }
    /// The same square seen from Black's side of the board.
    pub const fn flip_rank(self) -> Square {
        Square(self.0 ^ 56)
    }
}

impl std::str::FromStr for Square {
    type Err = ParseSquareError;
    fn from_str(s: &str) -> Result<Square, ParseSquareError> {
        match s.as_bytes() {
            &[f @ b'a'..=b'h', r @ b'1'..=b'8'] => Ok(Square::new(f - b'a', r - b'1')),
            _ => Err(ParseSquareError),
        }
    }
}
```

**End state.** `Square` with file, rank, flips, offsets that return `Option`
at the board edge, `Display` as `e4`, `FromStr`, and tests that round-trip all
64 squares.

#### 1.1.3 Bitboard

**What.** A set of squares in a `u64`, bit `i` set when square `i` is in the
set. Union, intersection and complement are single machine instructions, which
is why bitboard engines are fast.

**How.** A newtype with the bit operators implemented, `count()` from
`count_ones`, the lowest square from `trailing_zeros`, and iteration by
repeatedly clearing the lowest set bit. Shifts need file masks so a piece on
the h-file does not wrap to the a-file.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Bitboard(pub u64);

impl std::ops::BitAnd for Bitboard {
    type Output = Bitboard;
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl Iterator for Bitboard {
    type Item = Square;
    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }
        let square = Square::from_index(self.0.trailing_zeros() as u8);
        self.0 &= self.0 - 1; // clear the lowest set bit
        Some(square)
    }
}
```

`Iterator` on a `Copy` type means `for sq in bb` consumes a copy and leaves
`bb` untouched, which is usually what you want; know that it happens.

**End state.** `Bitboard` with the operators (`&`, `|`, `^`, `!`, and their
assign forms), `contains`, `count`, `lsb`, `more_than_one`, directional shifts,
file and rank constants, a `Debug` that prints an 8×8 grid, and tests.

#### 1.1.4 Move encoding

**What.** A move in 16 bits: 6 bits origin, 6 bits destination, 4 bits flags.
Sixteen bits matter later, because the transposition table (3.1) stores moves
and every byte of its entries costs cache.

**How.** A common flag layout: 0 quiet, 1 double pawn push, 2 king-side
castle, 3 queen-side castle, 4 capture, 5 en passant, 8–11 promotion to
knight, bishop, rook, queen, 12–15 the same promotions with capture. Then
"is capture" is `flag & 4 != 0` and "is promotion" is `flag & 8 != 0`.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Move(u16);

impl Move {
    pub const NULL: Move = Move(0);

    pub const fn new(from: Square, to: Square, flag: u16) -> Move {
        Move(from.index() as u16 | (to.index() as u16) << 6 | flag << 12)
    }
    pub const fn from(self) -> Square {
        Square::from_index((self.0 & 0x3f) as u8)
    }
    pub const fn to(self) -> Square {
        Square::from_index((self.0 >> 6 & 0x3f) as u8)
    }
}
```

**End state.** `Move` with constructors, flag queries, the promotion piece,
UCI text conversion (`e7e8q`; castling as the king's two-square move) and
tests.

#### 1.1.5 Castling rights

**How.** Four bits (white king side, white queen side, black king side, black
queen side). After any move, `rights &= MASK[from] & MASK[to]`, where `MASK`
is 64 entries of `0b1111` except the rook and king home squares: a move from
or to h1 clears the white king-side bit, from e1 clears both white bits. One
line then handles king moves, rook moves and rook captures.

**End state.** `CastlingRights` with queries, the update table and FEN
conversion (`KQkq`, `-`).

### 1.2 Attack generation

**What.** For every piece type and square, the set of squares it attacks given
the occupancy. Move generation, check detection, SEE, mobility and king safety
all read these tables, so they sit at the bottom of the engine.

#### 1.2.1 Leaper attack tables in `const fn`

**How.** Knights, kings and pawns attack a fixed pattern, so their tables are
computed once. Rust's `const` evaluation can build them at compile time with
`while` loops (no `for` in `const fn`).

```rust
const fn knight_attacks(sq: usize) -> u64 {
    let b = 1u64 << sq;
    let not_a = !FILE_A;
    let not_ab = !(FILE_A | FILE_B);
    let not_h = !FILE_H;
    let not_gh = !(FILE_G | FILE_H);
    (b << 17 & not_a) | (b << 15 & not_h) | (b << 10 & not_ab) | (b << 6 & not_gh)
        | (b >> 15 & not_a) | (b >> 17 & not_h) | (b >> 6 & not_ab) | (b >> 10 & not_gh)
}

pub const KNIGHT_ATTACKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        table[sq] = knight_attacks(sq);
        sq += 1;
    }
    table
};
```

**Rust you will practise.** What `const fn` may and may not do, `const`
blocks, operator precedence (`<<` binds tighter than `&`).

**End state.** Knight, king and pawn attack tables (pawns per colour), with
tests for corner and edge squares.

#### 1.2.2 Reference sliding attacks by ray walking

**What.** A slow, obviously correct function for bishop and rook attacks. It
generates the magic tables in 1.2.3 and is the oracle their tests compare
against.

```rust
fn sliding_attacks(sq: Square, occupied: u64, directions: &[(i8, i8)]) -> u64 {
    let mut attacks = 0;
    for &(df, dr) in directions {
        let (mut f, mut r) = (sq.file() as i8, sq.rank() as i8);
        loop {
            f += df;
            r += dr;
            if !(0..8).contains(&f) || !(0..8).contains(&r) {
                break;
            }
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if occupied & bit != 0 {
                break; // the first blocker is attacked, nothing behind it
            }
        }
    }
    attacks
}
```

**End state.** Reference bishop and rook attacks, tested on hand-built
positions.

#### 1.2.3 Magic bitboards

**What.** Constant-time slider attacks: multiply the relevant occupancy by a
per-square "magic" constant, shift, and use the result as an index into a
precomputed table.

**How it is usually done.**

1. For each square, the **mask** is the ray squares excluding the board edge
   (an edge blocker does not change the attack set).
2. Enumerate every subset of the mask with the Carry-Rippler trick and compute
   its attacks with the reference function.
3. **Find a magic** by trying sparse random numbers (the AND of three random
   `u64`s) until every subset maps to a slot that is empty or already holds the
   same attack set.
4. Store the magics as constants in the source once found, so start-up does
   not search for them. Build the attack tables at start-up or lazily.

```rust
struct Magic {
    mask: u64,
    magic: u64,
    shift: u32,    // 64 minus the number of mask bits
    offset: usize, // where this square's slots start in the shared table
}

impl Magic {
    fn index(&self, occupied: u64) -> usize {
        ((occupied & self.mask).wrapping_mul(self.magic) >> self.shift) as usize + self.offset
    }
}

// Carry-Rippler: visits every subset of `mask`, including the empty set, once.
let mut subset = 0u64;
loop {
    // ... record reference_attacks(sq, subset) at Magic::index(subset) ...
    subset = subset.wrapping_sub(mask) & mask;
    if subset == 0 {
        break;
    }
}
```

Write the magic finder as a separate binary (`src/bin/find_magics.rs` or an
example) that prints the constants. Build the tables with
`std::sync::LazyLock` first; its per-access check is measured in 8.2, where
you may move to an explicit initialisation.

**Rust you will practise.** Wrapping arithmetic, `LazyLock`/`OnceLock`,
`src/bin` targets, a small deterministic PRNG, property-style tests over random
occupancies.

**End state.** `bishop_attacks(sq, occ)` and `rook_attacks(sq, occ)` (queen as
their union) equal to the reference on every subset of every mask plus a
million random occupancies.

#### 1.2.4 Geometry tables: between and line

**What.** `between(a, b)`: squares strictly between two aligned squares
(empty otherwise). `line(a, b)`: the whole line through both, edge to edge.
Pins, check evasions and castling paths use them.

**How.** Build with the reference slider: for aligned `a` and `b`,
`between = attacks(a, occupancy = b) & attacks(b, occupancy = a)` using the
slider type that connects them.

**End state.** Two 64×64 tables with tests on known pairs.

### 1.3 Position state and FEN

#### 1.3.1 Board and state structures

**What.** One structure holding the position, plus the irreversible part of the
state (what `unmake` cannot recompute) in a separate `Copy` record that is
pushed on every move.

**How it is usually done.** Bitboards by piece type and by colour answer "where
are the knights"; a 64-entry mailbox answers "what is on e4" in one lookup.
Keep both and update them only through three primitives (`add_piece`,
`remove_piece`, `move_piece`) that also update hash keys; then they cannot
drift apart.

```rust
pub struct Board {
    pieces: [Bitboard; 6],        // by piece type
    colors: [Bitboard; 2],        // by colour
    mailbox: [Option<Piece>; 64],
    side_to_move: Color,
    state: State,                  // irreversible state of the current position
    history: Vec<State>,           // one entry per move made, for unmake and repetition
}

#[derive(Clone, Copy)]
struct State {
    hash: u64,
    castling: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u16,
    captured: Option<Piece>,
    checkers: Bitboard,
}
```

Decide the **en passant policy** now: store the en passant square only when a
pawn of the side to move can actually capture there. Otherwise two identical
positions differ in hash and in FEN, repetitions are missed and the table
(3.1) splits entries. Apply the same policy in FEN parsing, in `make` and in
hashing.

**Rust you will practise.** Struct layout, `Vec` with capacity reserved once
(`Vec::with_capacity`), keeping invariants private behind methods,
`std::mem::size_of` in a `const` assertion to watch the size:

```rust
const _: () = assert!(std::mem::size_of::<State>() <= 32);
```

**End state.** `Board` and `State` with the piece primitives, accessors
(`pieces(color, kind)`, `occupied()`, `piece_on(sq)`, `king_square(color)`) and
no public way to break the invariants.

#### 1.3.2 FEN parsing and writing

**How.** Split on whitespace into six fields, parse each, return a typed
error. Accept a missing halfmove and fullmove field (many tools omit them),
reject everything else that is malformed, and validate: one king each, no pawns
on the first or last rank, the side not to move is not in check.

```rust
#[derive(Debug)]
pub enum FenError {
    MissingField(&'static str),
    BadPlacement(String),
    BadPiece(char),
    BadSideToMove(String),
    BadCastling(String),
    BadEnPassant(String),
    BadNumber(String),
    IllegalPosition(&'static str),
}

impl std::fmt::Display for FenError { /* one message per variant */ }
impl std::error::Error for FenError {}
```

**Rust you will practise.** Custom error enums, `Display` plus
`std::error::Error`, the `?` operator, `str::split_ascii_whitespace`,
`char::to_digit`.

**End state.** `Board::from_fen` and `Board::to_fen`; a test that parses and
writes back several hundred FENs (take them from the perft suite in 1.8.2)
byte for byte.

#### 1.3.3 Consistency checker and pretty printer

**What.** A `validate()` that recomputes everything derivable (bitboards from
the mailbox, colour unions, king squares, hash keys, checkers) and compares.
It is the most useful debugging tool in the board phase.

**How.** Call it in `debug_assert!` after `make` and `unmake` during
development, and in tests always. `Display` prints the board, FEN and hash for
the `d` command in 2.1.4.

**End state.** `validate()` returns a `Result` naming the first inconsistency;
tests exercise it on corrupted boards.

### 1.4 Zobrist hashing

**What.** A 64-bit key per position, updated incrementally by XOR: one random
key per (piece, square), one for side to move, one per castling-rights value,
one per en passant file. Equal positions get equal keys; different positions
collide rarely. The key drives repetition detection, the transposition table,
the pawn table and correction histories.

#### 1.4.1 Key generation in const context

**How.** A deterministic PRNG in `const fn`, so keys are compile-time constants
and identical on every machine and every build.

```rust
const fn splitmix64(state: u64) -> (u64, u64) {
    let state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (state, z ^ (z >> 31))
}

pub const PIECE_SQUARE: [[u64; 64]; 12] = {
    let mut keys = [[0; 64]; 12];
    let mut state = 0x5748_4954_4553_5049; // any fixed seed
    let mut piece = 0;
    while piece < 12 {
        let mut sq = 0;
        while sq < 64 {
            let (next, key) = splitmix64(state);
            state = next;
            keys[piece][sq] = key;
            sq += 1;
        }
        piece += 1;
    }
    keys
};
```

**End state.** All key tables as constants; a test that no two keys are equal.

#### 1.4.2 Incremental hash and from-scratch verification

**How.** The piece primitives XOR piece-square keys; `make` XORs side,
castling (old rights out, new rights in) and en passant (old file out, new file
in). `compute_hash()` builds the key from scratch; `validate()` compares both.

**End state.** Incremental hash equal to the from-scratch hash after every move
of every perft test (checked in 1.8).

#### 1.4.3 Pawn, non-pawn and material keys

**What.** Extra keys maintained by the same primitives: a pawn key (pawns
only; the pawn hash table in 4.3.3 and pawn correction history in 6.3), a
non-pawn key per colour (6.3), and a material key (piece counts; the material
hash table in 4.3.2 and the endgame library in 4.14).

**How.** The pawn and non-pawn keys reuse the piece-square keys for the pieces
they include. The material key XORs `PIECE_SQUARE[piece][count]`, using the
piece count as the "square", so it depends only on counts.

**End state.** The three keys verified in `validate()`. This sub-step may wait
until 4.3 if you prefer; the board code is freshest in mind now.

### 1.5 Move generation

**The design choice.** Two families of generators exist:

- **Fully legal generation** computes check masks and pin rays and emits only
  legal moves. Perft is fastest this way.
- **Pseudo-legal generation plus a cheap legality test** emits moves that obey
  piece movement (and, when in check, only evasions), and tests legality just
  before a move is searched. Search usually benefits: many generated moves are
  pruned or never reached after a cutoff, so their legality test is never paid.

This plan uses the second, with pins and checkers precomputed per position so
the test is a few bit operations. Generation is split by type (**noisy**:
captures and queen promotions; **quiet**: the rest; **evasions**: when in
check) because the staged move picker (3.2) and quiescence (2.3.4) want them
separately.

#### 1.5.1 Allocation-free move list

**How.** A fixed array and a length. 218 is the largest known number of legal
moves in a position; 256 leaves room for pseudo-legal moves.

```rust
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}

impl MoveList {
    pub fn push(&mut self, mv: Move) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len] = mv;
        self.len += 1;
    }
}

impl std::ops::Deref for MoveList {
    type Target = [Move];
    fn deref(&self) -> &[Move] {
        &self.moves[..self.len]
    }
}
```

Let the caller own the list (`fn generate(&self, list: &mut MoveList)`)
instead of returning it: returning a 520-byte structure by value can cost a
memory copy per call when the optimiser cannot construct it in place, and move
generation runs at every node. Perft speed (1.8.4) shows whether it matters
on your build.

**Rust you will practise.** `Deref` to a slice (every slice method for free),
`Default` for arrays, `debug_assert!`, out-parameters versus return values.

**End state.** `MoveList`, and later a scored variant for the picker, with
tests.

#### 1.5.2 Attack queries, checkers and pins

**How.** `attackers_to(sq, occupied)` asks the reverse question: which pieces
of either colour attack `sq`? Place a knight on `sq` and intersect its attacks
with the knights, a bishop with bishops and queens, and so on. Checkers are
`attackers_to(king) & them`. Pinned pieces: slide from the king along rook
and bishop rays through enemy sliders; a line with exactly one piece between
king and slider pins that piece if it is ours.

```rust
let king = self.king_square(us);
let snipers = (ROOK_RAYS[king] & (their_rooks | their_queens))
    | (BISHOP_RAYS[king] & (their_bishops | their_queens));
let mut pinned = Bitboard(0);
for sniper in snipers {
    let blockers = between(king, sniper) & self.occupied();
    if blockers.count() == 1 {
        pinned |= blockers & self.color(us);
    }
}
```

Compute checkers and pinned pieces in `make` (or lazily, once per position)
and store them in `State`.

**End state.** `attackers_to`, `is_attacked(sq, by, occupied)`, `checkers()`,
`pinned()`, with tests on positions with pins, discovered checks and double
check.

#### 1.5.3 Pseudo-legal generation by type: noisy, quiet, evasions

**How.** For each piece type, intersect attacks with a target set: enemy pieces
for noisy moves, empty squares for quiet moves. Pawns are generated set-wise by
shifting the whole pawn bitboard. Castling checks rights, empty squares between
king and rook, and that the king does not start in, pass through or land on an
attacked square. When in check: king moves; if one checker, captures of the
checker and interpositions on `between(king, checker)`; if two checkers, king
moves only. Put queen promotions (with and without capture) in the noisy set
and under-promotions in the quiet set; that is the common choice, and it is a
decision to record.

**End state.** `generate(kind, list)` for `Noisy`, `Quiet`, `Evasions`, and
`All` as their union, with tests on positions with every special move.

#### 1.5.4 Legality test for pseudo-legal moves

**How.** With evasions generated when in check, only three kinds of move can
still be illegal: king moves onto attacked squares, moves of pinned pieces off
their pin line, and en passant (which can expose the king along a rank by
removing two pawns at once).

```rust
pub fn is_legal(&self, mv: Move) -> bool {
    let us = self.side_to_move;
    let from = mv.from();
    let king = self.king_square(us);
    if mv.is_en_passant() {
        return self.en_passant_is_legal(mv); // rare: simulate and test the king
    }
    if from == king {
        // castling was fully checked by the generator
        return mv.is_castle() || !self.is_attacked(mv.to(), !us, self.occupied() ^ from.bb());
    }
    // a pinned piece may move only along the line through its king
    !self.pinned().contains(from) || line(from, mv.to()).contains(king)
}
```

Note the king test removes the king from the occupancy: otherwise a king
stepping away from a rook along its ray still looks safe.

**End state.** `is_legal`; generated legal moves agree with the `chess` crate
(1.8.3).

#### 1.5.5 Validation of moves from tables (`is_pseudo_legal`)

**What.** The transposition table, killer slots and histories hand the search
moves that were legal in *some* position with a similar key or at a similar
ply. `is_pseudo_legal(mv)` decides whether `mv` could have been generated in
*this* position. Playing an unvalidated move corrupts the board.

**How.** Check the moving piece belongs to the side to move, the destination
does not hold a friendly piece, the flag matches (capture flag only with a
victim, en passant only on the en passant square, promotion only from the
seventh rank), the piece attacks or pushes to the destination, castling is
available, and when in check the move is an evasion.

**End state.** A fuzz test: for random positions and random 16-bit values,
`is_pseudo_legal` agrees exactly with "the move is in the generated list".

#### 1.5.6 `gives_check`

**How.** Direct check: the moved piece (or promoted piece) attacks the enemy
king from its destination. Discovered check: the moving piece was on a line
between the enemy king and one of our sliders, and leaves that line. Special
cases: en passant (two pieces leave), castling (the rook gives the check),
promotion. Precomputing, per position, "check squares" for each piece type
makes the direct test a single bit test.

**End state.** `gives_check(mv)` equal to "the opponent is in check after
`make`" for every move of the perft suite at depth 3.

### 1.6 Make and unmake

#### 1.6.1 State stack, make and unmake

**What.** `make_move` changes the board in place and pushes the previous
`State`; `unmake_move` restores the board and pops it. The alternative,
copy-make (copying the whole board per move), is simpler and not much slower
for a small board, but make/unmake keeps the history in one place for
repetition detection. Choose make/unmake here; you may measure copy-make later
if curious.

**How.** In `make`: push `state`; reset the en passant key; remove a captured
piece (en passant victim on another square); move the piece; handle the rook
for castling and the piece change for promotion; update castling rights through
the mask table; set a new en passant square only under the policy of 1.3.1;
update halfmove clock; flip the side; compute checkers and pins. In `unmake`:
flip back, move pieces back using the move and `state.captured`, then pop the
saved state.

```rust
pub fn unmake_move(&mut self, mv: Move) {
    self.side_to_move = !self.side_to_move;
    let captured = self.state.captured;  // read before the state is replaced
    // ... move the piece back, undo promotion, castling rook, put `captured` back ...
    self.state = self.history.pop().expect("unmake without a matching make");
}
```

**Rust you will practise.** `Option::take`, `expect` with a message that names
the broken invariant, borrow rules when a method needs both `&mut self` and a
table (split into small private methods).

**End state.** For every position in the perft suite, `make` then `unmake`
restores a board equal to the original (`#[derive(PartialEq)]` on the board for
tests).

#### 1.6.2 Null move

**What.** "Pass": flip the side to move, clear the en passant square, update
the hash, keep pieces. Used by null move pruning (3.5).

**End state.** `make_null_move` and `unmake_null_move`; never called when in
check (a `debug_assert!`); hash and state restored exactly.

### 1.7 Game rules: repetition, fifty moves, insufficient material

**What.** Draw detection the search can afford at every node.

**How it is usually done.**

- **Repetition.** Positions can repeat only since the last capture or pawn
  move (the halfmove clock), and only with the same side to move, so compare
  keys 4, 6, 8, … plies back, never further than the halfmove clock.

  ```rust
  pub fn is_repetition(&self) -> bool {
      let n = self.history.len();
      let reach = (self.state.halfmove_clock as usize).min(n);
      (4..=reach)
          .step_by(2)
          .any(|back| self.history[n - back].hash == self.state.hash)
  }
  ```

  In search, one earlier occurrence inside the search tree is enough to call
  the position a draw (the side can repeat again); positions from the game
  before the root need two. Null moves break a repetition chain; decide how the
  scan treats them.
- **Fifty moves.** Halfmove clock at 100 or more is a draw, unless the side to
  move is checkmated (check mate first).
- **Insufficient material.** King against king, king and a minor against a
  king, and kings with bishops all on one colour. KNN against K is not a draw
  by rule; score it through evaluation later.

**End state.** `is_repetition`, `is_fifty_move_draw`,
`is_insufficient_material`, each tested; the halfmove clock stays correct
through make/unmake.

### 1.8 Perft and the correctness suite

**What.** Perft counts the leaf nodes of the legal move tree to a fixed depth.
Every generator, make/unmake and legality bug shows up as a wrong count, and
"divide" (count per root move) finds the move where it hides.

#### 1.8.1 `perft` and divide

```rust
pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut moves = MoveList::default();
    board.generate(GenKind::All, &mut moves);
    let mut nodes = 0;
    for &mv in moves.iter() {
        if !board.is_legal(mv) {
            continue;
        }
        board.make_move(mv);
        nodes += perft(board, depth - 1);
        board.unmake_move(mv);
    }
    nodes
}
```

At depth 1 you can count legal moves without making them ("bulk counting");
keep a switch so the slow path stays testable.

**End state.** `perft` and `divide`, reachable from UCI in 2.1.4.

#### 1.8.2 Reference positions and the EPD suite

**How.** The Chess Programming Wiki's six standard positions with their known
counts, as ordinary tests at depths that run in seconds, plus deeper counts as
`#[ignore]` tests you run before a release (`cargo test --release -- --ignored`).

| Position | FEN | Counts by depth |
|---|---|---|
| Start | `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1` | 20, 400, 8,902, 197,281, 4,865,609, 119,060,324 |
| Kiwipete | `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1` | 48, 2,039, 97,862, 4,085,603, 193,690,690 |
| 3 | `8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1` | 14, 191, 2,812, 43,238, 674,624, 11,030,083 |
| 4 | `r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1` | 6, 264, 9,467, 422,333, 15,833,292 |
| 5 | `rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8` | 44, 1,486, 62,379, 2,103,487, 89,941,194 |
| 6 | `r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/3P1N1P/PPP1NPP1/R4RK1 w - - 0 10` | 46, 2,079, 89,890, 3,894,594, 164,075,551 |

Add a larger EPD perft suite (for example the widely circulated `perftsuite.epd`
with about 130 positions) stored under `tests/data/`, parsed by the test.

**End state.** All six positions exact to depth 5 in release, the EPD suite
exact to depth 4, shallow versions in CI (0.4).

#### 1.8.3 Differential testing against the `chess` crate

**What.** While the `chess` crate is still a dependency, use it as an
independent oracle. Perft counts only prove totals; comparing move *sets*
position by position over random games finds errors that cancel out in totals.

**How.** Play thousands of random games with a seeded PRNG; at every ply
compare the sorted UCI strings of both engines' legal moves, the check status,
and make/unmake restoration; pick the next move from the agreed list.

```rust
#[test]
fn legal_moves_match_the_chess_crate() {
    let mut rng = XorShift64::new(0xC0FFEE);
    for _game in 0..2_000 {
        let mut ours = Board::starting_position();
        let mut reference = chess::Board::default();
        for _ply in 0..300 {
            let mut mine: Vec<String> = ours.legal_moves().iter().map(|m| m.to_string()).collect();
            let mut theirs: Vec<String> =
                chess::MoveGen::new_legal(&reference).map(|m| m.to_string()).collect();
            mine.sort();
            theirs.sort();
            assert_eq!(mine, theirs, "position {}", ours.to_fen());
            // ... stop at game end; otherwise pick one move by rng and play it on both boards ...
        }
    }
}
```

**End state.** The differential test passes over at least a million
positions (an `#[ignore]` long version) and a few thousand in the ordinary
suite.

#### 1.8.4 Board speed baseline

**What.** A number to compare against later, not a target.

**How.** Time perft 6 from the start position and perft 5 from Kiwipete in
release, three runs each, on an idle host; report nodes per second with and
without bulk counting.

**End state.** An observation entry in `EXPERIMENTS.md` with the commit,
binary hash and timings.

### 1.9 Static exchange evaluation

**What.** SEE answers "if both sides keep capturing on this square with their
cheapest attacker, who comes out ahead, and by how much?" without making moves.
Consumers: capture ordering (3.2), pruning of bad captures in quiescence (3.9)
and in the main search (3.8), and ProbCut (6.6). Write it now, because it
belongs to the board and needs the attack queries fresh in mind.

**How it is usually done.** The swap algorithm. Put the first victim's value in
`gain[0]`. Repeatedly remove the last attacker from the occupancy (which
reveals sliders behind it, the x-rays), find the other side's least valuable
attacker, and record the speculative balance. Then fold the list back from the
end, because either side may stop capturing when continuing loses.

```rust
// gain[d]: material balance for the side that captures at depth d,
// assuming it is recaptured.
gain[d] = value(attacker_at_depth_d) - gain[d - 1];
if gain[d].max(-gain[d - 1]) < 0 {
    break; // neither side can improve by continuing
}
// ... after the loop, fold back ...
while d > 1 {
    d -= 1;
    gain[d - 1] = -(-gain[d - 1]).max(gain[d]);
}
```

Most engines expose the threshold form `see_ge(mv, threshold) -> bool`, which
stops as soon as the answer is known and is what pruning needs. Handle en
passant (the victim is not on the destination), promotions (the piece changes
value) and the king as an attacker (it may capture last only if nothing
defends). Pinned attackers are the classic simplification: a pinned piece
cannot really take part in the exchange, but excluding it costs a pin test per
attacker. Ignoring pins is common and cheap; handling them makes SEE right in
a few tactical positions and is measurable by games. Record your choice.

**End state.** `see_ge` with tests from a table of positions, moves, thresholds
and expected answers, including x-rays, en passant, promotions and king
captures.

---

## Phase 2 — Engine rebuilt on the new board (release 2.0.0)

**Why now.** The board is proven; the engine around it has the defects of the
audit. This phase rewrites the protocol layer, the search thread, scores and a
deliberately *minimal* search on the new board, keeps 1.4.0's evaluation for
one more release so the gate measures the rewrite rather than a new evaluation,
and ends with the `chess` crate gone.

**End state of the phase.** Whitespine 2.0.0: zero dependencies, a correct and
complete UCI surface, a search that never loses a command or plays a random
move, integer scores with correct mates, a bench fingerprint, and a registered
SPRT win over 1.4.0.

### 2.1 UCI protocol layer

**Rust you will practise.** Enums with data, `FromStr`, iterators over tokens,
generic parsing helpers with trait bounds, `std::io::BufRead::lines`,
locking `stdout` once per message.

#### 2.1.1 Typed command parsing

**What.** Parse each input line into a value of one enum, separately from
executing it. Parsing becomes testable without an engine, and execution becomes
one `match`.

```rust
pub enum UciCommand {
    Uci,
    IsReady,
    UciNewGame,
    Position { fen: Option<String>, moves: Vec<String> },
    Go(SearchLimits),
    Stop,
    PonderHit,
    SetOption { name: String, value: Option<String> },
    Quit,
    // extensions for development
    Display,
    Eval,
    Perft(u32),
    Bench(Option<u32>),
}

impl std::str::FromStr for UciCommand {
    type Err = UciError;
    fn from_str(line: &str) -> Result<UciCommand, UciError> {
        let mut tokens = line.split_whitespace();
        match tokens.next() {
            Some("uci") => Ok(UciCommand::Uci),
            Some("go") => SearchLimits::parse(tokens).map(UciCommand::Go),
            // ...
            Some(other) => Err(UciError::UnknownCommand(other.to_string())),
            None => Err(UciError::Empty),
        }
    }
}
```

Unknown commands and errors are reported as `info string …` and otherwise
ignored, as the protocol asks. `position` with an illegal move reports the move
and keeps the position up to the last legal move or rejects the whole command;
pick one rule and state it.

**End state.** `UciCommand` with unit tests covering every command, extra
whitespace, `position fen` with and without `moves`, and malformed lines.

#### 2.1.2 Search limits: every `go` parameter

**How.** Parse `wtime btime winc binc movestogo depth nodes mate movetime
infinite ponder searchmoves`. Some GUIs send a negative `wtime` when the clock
has run out; parse times as signed and clamp at zero. Parameters the engine
does not implement yet (`mate`, `searchmoves` until 6.8, `ponder` until 7.2)
are parsed and ignored explicitly, never swallowed by accident.

```rust
fn next_value<'a, T: std::str::FromStr>(
    tokens: &mut impl Iterator<Item = &'a str>,
    name: &'static str,
) -> Result<T, UciError> {
    tokens
        .next()
        .and_then(|token| token.parse().ok())
        .ok_or(UciError::BadValue(name))
}
```

**End state.** `SearchLimits` fully parsed and tested; `go nodes 1000` stops
after about 1,000 nodes; `go` with no parameters means infinite, not depth 2
(W-10).

#### 2.1.3 Option registry

**What.** One table describing every UCI option (name, type, default, bounds),
used both to print `option name …` lines and to validate `setoption`. Option
names are case-insensitive.

**How.** A slice of descriptors and one `apply` function. Start with `Hash`
(spin, MB; wired in 3.1), `Threads` (spin, 1–1 until 7.3), `Move Overhead`
(spin, ms). Later phases add `Clear Hash` (button), `MultiPV` (6.8), `Ponder`
(7.2), `SyzygyPath` and `SyzygyProbeDepth` (4.13).

```rust
pub enum OptionKind {
    Spin { default: i64, min: i64, max: i64 },
    Check { default: bool },
    Str { default: &'static str },
    Button,
}

pub struct OptionSpec {
    pub name: &'static str,
    pub kind: OptionKind,
}
```

**End state.** `uci` prints options from the registry; `setoption` with an
unknown name or an out-of-range value answers `info string` and changes
nothing (W-15).

#### 2.1.4 Debug commands and command-line `bench`

**What.** `d` prints board, FEN, hash and checkers; `eval` prints the static
evaluation (later with a per-term breakdown); `go perft N` prints divide and
the total; `bench [depth]` runs the benchmark of 2.5. Running
`whitespine bench` from the command line runs the benchmark and exits, which CI
(0.4) and PGO (8.4) need. Remove the start-up banner; GUIs expect the engine to
be silent until `uci`.

**End state.** All four commands available both from UCI and, for `bench`, as a
program argument.

#### 2.1.5 fastchess compliance check

**How.** `fastchess --compliance D:\path\to\whitespine.exe` runs a scripted
protocol session against the engine.

**End state.** The compliance check passes; the run is recorded with the
binary's hash.

### 2.2 Search thread and stop control

#### 2.2.1 Worker thread, atomic stop, no lost commands

**What.** The UCI thread must keep reading input during a search (to see
`stop`, `isready`, `quit`), and the search must learn about `stop` without
reading the command channel (W-07).

**How it is usually done.** A long-lived search worker owns the search state
that survives between moves (transposition table, histories) and waits for
jobs. A shared `AtomicBool` is the only thing the search polls. The UCI thread
sets it on `stop` and `quit`, and waits for the running search to finish before
starting another `go` or changing the position.

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let stop = Arc::new(AtomicBool::new(false));
let worker_stop = Arc::clone(&stop);
let (jobs, job_receiver) = std::sync::mpsc::channel::<SearchJob>();

let worker = std::thread::spawn(move || {
    let mut searcher = Searcher::new(worker_stop);
    for job in job_receiver {
        searcher.run(job); // prints info lines and exactly one bestmove
    }
});

// UCI thread, on "stop":
stop.store(true, Ordering::Relaxed);
```

To wait for the end of a search, let `run` signal completion (a second channel,
or a `Mutex<bool>` with a `Condvar`), so a `go` that arrives while a search is
running is queued rather than lost.

**Rust you will practise.** `Arc`, atomics and `Ordering`, `mpsc` channels,
`move` closures, `JoinHandle`, `Condvar`, why `Relaxed` is fine for a stop flag.

**End state.** `isready` answers immediately during a search; `go`, `go`,
`stop`, `stop` produces two `bestmove` lines; `quit` during a search exits
cleanly.

#### 2.2.2 Clock and node checks

**How.** Take `Instant::now()` when `go` is **parsed** on the UCI thread and
pass it with the job. The GUI's clock started when it sent `go`; any delay
before the worker dequeues the job (a previous search finishing, a table being
cleared) is already spent, and a clock started later than that is a hidden
time overrun that shows up as forfeits at fast controls. The search checks the
stop flag and the clock every 1,024 or 2,048 nodes, not at every node.

```rust
fn should_stop(&mut self) -> bool {
    if self.nodes & 2047 == 0
        && (self.stop.load(Ordering::Relaxed) || self.clock.hard_limit_reached(self.nodes))
    {
        self.stopped = true;
    }
    self.stopped
}
```

**End state.** Node and time limits honoured within one check interval.

#### 2.2.3 Output: info lines and an always-legal `bestmove`

**How.** After each completed iteration print `info depth seldepth score nodes
nps time pv` (later `hashfull`, `multipv`, `tbhits`). Always complete depth 1
before honouring a stop (it takes microseconds), so a best move always exists
(W-08). If the root has no legal move, print `bestmove 0000`; that is the only
case (W-02). A claimable draw at the root is not a reason to stop playing.

**End state.** A fuzz-style test that sends `go` followed immediately by
`stop` a thousand times on random positions, and checks every `bestmove` is
legal.

### 2.3 Scores and the minimal search

#### 2.3.1 Integer scores and mate distance by ply

**What.** Scores are `i32` in centipawns. Mates are encoded by distance from
the root, so the same mate has the same score at every depth and a shorter mate
always scores higher (W-05).

```rust
pub const INFINITE: i32 = 32_001;
pub const MATE: i32 = 32_000;
pub const MAX_PLY: i32 = 128;
pub const MATE_IN_MAX_PLY: i32 = MATE - MAX_PLY;

/// Score for the side to move when it is checkmated at `ply`.
pub const fn mated_in(ply: i32) -> i32 {
    -MATE + ply
}

pub fn uci_score(score: i32) -> String {
    if score >= MATE_IN_MAX_PLY {
        format!("mate {}", (MATE - score + 1) / 2)
    } else if score <= -MATE_IN_MAX_PLY {
        format!("mate -{}", (MATE + score) / 2)
    } else {
        format!("cp {score}")
    }
}
```

Keep scores inside `i16` range (they will be stored as `i16` in the table);
evaluation must never return a value in the mate range.

**End state.** Score constants and conversions with tests: mate in 1, 2 and 3,
mated in 1 and 2.

#### 2.3.2 Port the 1.4.0 evaluation with a differential test

**What.** Rewrite 1.4.0's heuristic on the new board in integers, once, so the
2.7 gate measures the rewrite and not an evaluation change. Replacing the
evaluation is Phase 4's job.

**How.** Round the `f64` terms consistently (for example, compute in `f64`
exactly as before and round at the end) and test against the old code on
random positions while both boards exist.

**End state.** Over 100,000 random positions, the new evaluation equals the old
one rounded, within 1 cp; the old heuristic module is then deleted.

#### 2.3.3 Alpha-beta, iterative deepening and the PV table

**What.** The minimal search: negamax with fail-soft alpha-beta, iterative
deepening, draw detection, mate scores by ply and a principal variation table.
No pruning yet; Phase 3 adds everything else one feature at a time.

**How it is usually done.** *Fail-soft* returns the best score found even when
it lies outside the window, which later lets the transposition table store
tighter bounds (W-13). The *triangular PV table* keeps one line per ply and
copies the child's line up when a move raises alpha, instead of allocating
vectors.

```rust
// inside the move loop of negamax
board.make_move(mv);
let score = -self.negamax(board, depth - 1, ply + 1, -beta, -alpha);
board.unmake_move(mv);
if self.stopped {
    return 0;
}
if score > best_score {
    best_score = score;
    if score > alpha {
        alpha = score;
        self.update_pv(ply, mv);
        if score >= beta {
            break; // fail-soft: return best_score, which may exceed beta
        }
    }
}
// after the loop, with no legal move:
// if in check { mated_in(ply) } else { 0 }

fn update_pv(&mut self, ply: usize, mv: Move) {
    self.pv[ply][ply] = mv;
    for i in ply + 1..self.pv_len[ply + 1] {
        self.pv[ply][i] = self.pv[ply + 1][i];
    }
    self.pv_len[ply] = self.pv_len[ply + 1];
}
```

At the root, keep the best move of the last *completed* iteration. A partially
searched iteration may be used only if its best move was already searched with
a better score than the previous iteration's; keep it simple first.

**End state.** A search that finds mate in 2 and 3 at the expected depths,
reports `score mate N`, respects repetition and fifty-move draws, and never
allocates per node.

#### 2.3.4 Quiescence search done right

**What.** At depth zero, continue with captures until the position is quiet so
the evaluation is not taken in the middle of an exchange (the horizon effect).

**How it is usually done.** If not in check, the side to move may "stand pat"
(take the static evaluation) because it is not forced to capture; return at
once if that already reaches beta. If **in check**, standing pat is not
allowed: search all evasions, and return a mate score if there are none (W-03).
Generate captures (and queen promotions) only; no quiet checks at this stage
(W-04). Order by MVV-LVA (2.3.5).

**End state.** Quiescence with a tactical test: positions where a piece hangs
to a two-move combination at the horizon are evaluated correctly; mates in
quiescence are found.

#### 2.3.5 MVV-LVA ordering

**What.** Most Valuable Victim, Least Valuable Attacker: `victim × 16 −
attacker` over small integer piece values, with the king as the largest
attacker value rather than infinity (W-11). Promotions score as captures of
the promotion piece's value.

**End state.** Captures sorted by selection (pick the best remaining move each
time; it is cheaper than a full sort when a cutoff comes early).

### 2.4 Time management v1

**What.** A budget that never goes negative, spends more when there is more,
and always leaves the overhead (W-09).

**How it is usually done.** A *soft* limit, checked after each iteration (do
not start a new iteration past it), and a *hard* limit, checked in the search
(abort). Divide the remaining time by the moves left (`movestogo` or an
estimate), add most of the increment, subtract the overhead, and cap both
limits by a fraction of the remaining time.

```rust
pub fn budget(time_ms: u64, increment_ms: u64, moves_to_go: Option<u64>, overhead_ms: u64) -> Budget {
    let available = time_ms.saturating_sub(overhead_ms);
    let moves_left = moves_to_go.unwrap_or(30).clamp(1, 50);
    let soft = available / moves_left + increment_ms * 3 / 4;
    let hard = (soft * 3).min(available * 3 / 4);
    Budget { soft: soft.min(hard), hard }
}
```

`movetime` uses `movetime − overhead` as both limits; `infinite`, `depth` and
`nodes` without clock set no time limit.

**End state.** Zero time losses in a 1,000-game self-play match at 1+0.01, and
at 40 moves in 10 seconds repeating (`tc=40/10`; confirm the unit in
`fastchess -help` before trusting the run).

### 2.5 Bench command and node fingerprint

**What.** A fixed search over a fixed set of positions that prints the total
node count and NPS. The node count is the engine's **fingerprint**: any change
that should not alter behaviour must reproduce it exactly; any change that
alters it is a behaviour change.

**How it is usually done.** 40–50 positions from varied phases (take them from
your own games and the perft suite; include a few endgames and checks), a
fixed depth, a fresh table and histories for each position, one thread. Print
the total last, in the form other tools parse: `<nodes> nodes <nps> nps`.

```rust
pub fn bench(depth: i32) {
    let start = std::time::Instant::now();
    let mut nodes = 0u64;
    for fen in BENCH_POSITIONS {
        let mut searcher = Searcher::for_bench();
        nodes += searcher.search_fixed_depth(fen, depth);
    }
    let elapsed_ms = start.elapsed().as_millis().max(1) as u64;
    println!("{nodes} nodes {} nps", nodes * 1000 / elapsed_ms);
}
```

Record the fingerprint in the commit message of every engine change that moves
it, and in every `EXPERIMENTS.md` entry. When a feature changes the tree, note
the new count; when a refactor keeps it, say so.

**End state.** `whitespine bench` deterministic across runs, debug and release
builds, and the three CI operating systems. CI prints it (0.4).

### 2.6 Remove the `chess` crate

**What.** Delete the dependency, the old modules and the differential tests
that needed it; keep the perft suites and the recorded differential results.

**End state.** `[dependencies]` empty; `cargo tree` shows only `whitespine`;
all tests pass; bench fingerprint unchanged by the removal.

### 2.7 Gate and release 2.0.0

**What.** The rewrite gate against 1.4.1 (the bug-fixed old engine) and a
checkpoint against Tier 1.

**How.** First run the UCI regression tests from 0.6.1 against the 2.0.0
binary: every one must pass, which proves no audit bug came back with the
rewrite. Then register in `EXPERIMENTS.md`: baseline Whitespine 1.4.1 (tag
`v1.4.1`), candidate the 2.0.0 release candidate, SPRT `[0, 10]`, 8+0.08,
prediction (a large gain from speed alone, since W-01 made the old engine
several times slower late in games). Then a Tier-1 gauntlet at 10+0.1 with the
balanced book, 100 games per opponent. Release when you are satisfied.

```powershell
& D:\chess\whitespine-lab\bin\fastchess.exe `
  -engine cmd=D:\chess\whitespine-lab\bin\whitespine-2.0.0.exe name=ws-2.0.0 `
  -engine cmd=D:\chess\whitespine-lab\bin\whitespine-1.4.1.exe name=ws-1.4.1 `
  -each tc=8+0.08 option.Threads=1 `
  -openings file=D:\chess\whitespine-lab\books\UHO_Lichess_4852_v1.epd format=epd order=random `
  -repeat -rounds 100000 -concurrency 14 -use-affinity `
  -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 model=normalized `
  -pgnout file=D:\chess\whitespine-lab\results\WS-003\games.pgn
```

(1.4.1 has no `Hash` option, so none is set here.)

**End state.** All 0.6.1 regression tests pass on 2.0.0; SPRT accepted and
recorded; Tier-1 ratings re-measured; version 2.0.0 tagged and released;
README updated.

### 2.8 Fast-control readiness: SPRTs at 3+0.03

**What.** Make the engine reliable at 3+0.03 with 14 concurrent games, and
move this plan's default to it. A shorter control plays about 2.5 times as
many games per hour as 8+0.08, which matters once gains shrink and SPRTs need
tens of thousands of games.

**How it is usually done.** At 3+0.03 an engine has about 75 ms per move early
in a game and a 30 ms increment later. What breaks at such controls:

- **Latency.** Time between receiving `go` and starting the search, and
  between finishing and printing `bestmove`: parse without allocation-heavy
  work, flush output once per line, never block the UCI thread. fastchess's
  `-show-latency` reports it.
- **Clock granularity.** Check the clock every 1,024 nodes; at a few million
  nodes per second that is well under a millisecond.
- **Minimum move time and overhead.** The budget of 2.4 must leave the
  `Move Overhead` (default 10 ms) plus a margin even when the increment is the
  whole budget. The overhead is thrown away on every move, so a generous one
  costs measurable strength at a control where a move has 30–75 ms; size the
  margin from measured forfeits, not by fear.
- **`ucinewgame` cost.** Clearing a large table between games takes time; with
  `Hash=16` it is negligible, but measure it.
- **Machine load.** 14 engines on 16 physical cores leave room for fastchess;
  keep everything else off the host.

**End state.** A 2,000-game self-play run of the 2.0.0 head at 3+0.03,
concurrency 14, `-use-affinity`, with **zero** time forfeits and no latency
warnings, recorded in `EXPERIMENTS.md`; from here on every SPRT and SPSA runs
at 3+0.03 unless its registration says otherwise, with long-control
confirmations at 10+0.1. If forfeits appear later (after 7.1 or a
new feature), the default returns to 8+0.08 until they are fixed.

```powershell
& D:\chess\whitespine-lab\bin\fastchess.exe `
  -engine cmd=D:\chess\whitespine-lab\bin\whitespine-2.0.0.exe name=a `
  -engine cmd=D:\chess\whitespine-lab\bin\whitespine-2.0.0.exe name=b `
  -each tc=3+0.03 option.Hash=16 option.Threads=1 `
  -openings file=D:\chess\whitespine-lab\books\UHO_Lichess_4852_v1.epd format=epd order=random `
  -repeat -rounds 1000 -concurrency 14 -use-affinity -show-latency `
  -pgnout file=D:\chess\whitespine-lab\results\WS-004\games.pgn `
  -log file=D:\chess\whitespine-lab\results\WS-004\fastchess.log level=warn
```

---

## Phase 3 — Search fundamentals

**Why this order.** Each feature makes the next one pay. The transposition
table gives every iteration the previous best move; good ordering makes cutoffs
come early; PVS and null-window searches rely on good ordering; null move,
reverse futility and LMR rely on PVS and on a stable static evaluation; move-loop
pruning relies on LMR's ordering assumptions; aspiration windows need a stable
score. The evaluation stays 1.4.0's until Phase 4, so margins chosen here are
provisional and are fitted later (5.7, 6.9, 8.5). All SPRTs in this phase run
at 3+0.03 (2.8).

**How each step is gated.** Implement; run the unit tests and bench (a new
fingerprint is expected); screen with a tactical suite at fixed nodes (for
example WAC, 300 positions: solved count is a screen, not a verdict); register
an SPRT against the previous accepted head, `[0, 10]` for the transposition
table, null move and LMR, `[0, 5]` for the rest; record the result. A rejected
feature is reverted or reworked, not accumulated.

**Rust you will practise.** Associated constants and zero-sized types for node
types, heap-allocated large arrays (a history table built on the stack
overflows it in debug builds), `Box<[T]>`, `LazyLock` for tables computed with
floating point, careful integer arithmetic.

**End state of the phase.** A conventional modern alpha-beta core with every
constant tunable, fitted once by SPSA, and a Tier-2 checkpoint.

### 3.1 Transposition table

**What.** A large hash table keyed by Zobrist key, storing for each searched
position its best move, score, depth and bound type. It lets transpositions
share work, and above all gives every node the best move from an earlier
search of the same position (W-06).

#### 3.1.1 Entry, table and indexing

**How it is usually done.** A small `Copy` entry holding part of the key for
verification, a 16-bit move, a 16-bit score, the depth and the bound (exact,
lower, upper). Size the table from `Hash` in megabytes. Index with the high
bits of `key × len` (Lemire's multiply-shift), which works for any length, not
only powers of two.

```rust
#[derive(Clone, Copy, Default)]
pub struct Entry {
    key: u32,     // upper half of the Zobrist key
    mv: Move,
    score: i16,
    depth: u8,
    bound: u8,    // 0 none, 1 upper, 2 lower, 3 exact
}

fn index(&self, key: u64) -> usize {
    ((key as u128 * self.entries.len() as u128) >> 64) as usize
}
```

Start with one entry per slot; buckets, stored static evaluation and ageing
come in 6.1.

#### 3.1.2 Probe, store, replacement and mate adjustment

**How.** Store mate scores relative to the node, not the root, and convert
back on probe; otherwise a mate found at ply 10 is reported from ply 3 with the
wrong distance.

```rust
fn score_to_tt(score: i32, ply: i32) -> i32 {
    if score >= MATE_IN_MAX_PLY {
        score + ply
    } else if score <= -MATE_IN_MAX_PLY {
        score - ply
    } else {
        score
    }
}
// score_from_tt is the inverse: subtract ply for wins, add it for losses.
```

Replacement: always replace when the key differs; when it matches, replace if
the new depth is not much lower or the new bound is exact; keep the old move if
the new store has none. Implement `hashfull` (permille of the first 1,000
slots in use) and clearing on `ucinewgame` and `Clear Hash`.

#### 3.1.3 Search integration and the Hash option

**How.** Probe at the start of every node after draw detection. At non-PV
nodes, when the stored depth is at least the current depth and the bound allows
it, return the stored score. Always use the stored move first in ordering, after
`is_pseudo_legal` and `is_legal` checks (1.5.5). Store after the move loop with
the bound decided by the result against the original alpha and beta.

```rust
if !pv_node && entry.depth as i32 >= depth {
    let score = score_from_tt(entry.score as i32, ply);
    let usable = match entry.bound {
        EXACT => true,
        LOWER => score >= beta,
        UPPER => score <= alpha,
        _ => false,
    };
    if usable {
        return score;
    }
}
```

Interactions to know about: a table cutoff can hide a repetition or the
fifty-move rule (the stored score came from a different history), so skip
cutoffs at the root and consider skipping them when the halfmove clock is high.

**End state.** A tested table (store/probe round trip, mate adjustment,
replacement, resize) used by the search; `Hash` and `Clear Hash` options;
`hashfull` reported; SPRT `[0, 10]` accepted.

### 3.2 Staged move picker, killers and quiet history

**What.** Search the moves most likely to cause a cutoff first, and generate
moves lazily so that a cutoff on the table move costs no generation at all.

#### 3.2.1 Staged picker with SEE-split captures

**How it is usually done.** A state machine: table move → generate captures,
score by MVV-LVA → captures with `see_ge(mv, 0)` → killers → generate quiets,
score by history → quiets → the captures that lost SEE. Skip moves already
returned (the table move, killers) in later stages.

```rust
enum Stage {
    TableMove,
    GenerateNoisy,
    GoodNoisy,
    Killers,
    GenerateQuiet,
    Quiet,
    BadNoisy,
    Done,
}

pub fn next(&mut self, board: &Board, history: &History) -> Option<Move> {
    loop {
        match self.stage {
            Stage::TableMove => {
                self.stage = Stage::GenerateNoisy;
                if self.table_move != Move::NULL && board.is_pseudo_legal(self.table_move) {
                    return Some(self.table_move);
                }
            }
            Stage::GoodNoisy => match self.pick_best_noisy() {
                Some(mv) if mv == self.table_move => continue,
                Some(mv) if !board.see_ge(mv, 0) => self.bad_noisy.push(mv),
                Some(mv) => return Some(mv),
                None => self.stage = Stage::Killers,
            },
            // ... the other stages ...
            Stage::Done => return None,
        }
    }
}
```

**End state.** A picker with a test that, for random positions, it returns
every legal move exactly once (after legality filtering) and the table move
first. Quiescence uses the same picker with a noisy-only mode.

#### 3.2.2 Killer moves

**What.** Two quiet moves per ply that caused a beta cutoff recently at the
same ply in sibling nodes; they often refute the sibling too.

**End state.** Killers stored on quiet cutoffs, cleared for the child ply at
node entry, tried after good captures.

#### 3.2.3 Quiet history with gravity

**What.** A `[color][from][to]` table of scores raised for quiet moves that
cause cutoffs and lowered for quiet moves searched before them without a
cutoff. *Gravity* keeps values bounded and lets old information fade.

```rust
const HISTORY_MAX: i32 = 16_384;

fn apply_bonus(entry: &mut i16, bonus: i32) {
    let bonus = bonus.clamp(-HISTORY_MAX, HISTORY_MAX);
    let value = *entry as i32;
    *entry = (value + bonus - value * bonus.abs() / HISTORY_MAX) as i16;
}
// on a cutoff by quiet move m at depth d: bonus = (d * d).min(1_200) for m,
// and the same value as a malus for each quiet searched before m.
```

**End state.** History used for quiet ordering; SPRT `[0, 5]` for 3.2 as a
whole (picker, killers and history land together because the picker's value
comes from its scores).

### 3.3 Principal variation search and node types

**What.** With good ordering the first move is usually best. Search it with the
full window, the rest with a null window `(alpha, alpha + 1)` that can only
answer "better or not", and re-search with the full window only when a null
window search says "better".

**How it is usually done.** Distinguish PV nodes (full window) from non-PV
nodes at compile time. Zero-sized marker types with an associated constant let
the compiler generate two specialised copies of the search with no runtime
flag.

```rust
trait NodeType {
    const PV: bool;
}
struct Pv;
struct NonPv;
impl NodeType for Pv {
    const PV: bool = true;
}
impl NodeType for NonPv {
    const PV: bool = false;
}

let score = if move_count == 1 {
    -self.negamax::<Pv>(board, depth - 1, ply + 1, -beta, -alpha)
} else {
    let s = -self.negamax::<NonPv>(board, depth - 1, ply + 1, -alpha - 1, -alpha);
    if s > alpha && s < beta {
        -self.negamax::<Pv>(board, depth - 1, ply + 1, -beta, -alpha)
    } else {
        s
    }
};
```

**Rust you will practise.** Generic functions over marker types, associated
constants, monomorphisation, reading the result in a profiler later.

**End state.** PVS with node types; the table cutoff rule uses `N::PV`; SPRT
`[0, 5]`.

### 3.4 Search stack and the improving flag

**What.** A per-ply array carrying what a node needs to know about its
ancestors: static evaluation, the move made, whether it was a null move, killers
and, later, the excluded move (6.5) and continuation-history references (6.2).
*Improving* is "the static evaluation is better than two plies ago"; pruning
margins depend on it.

**How.** A fixed array of `MAX_PLY + some` entries owned by the searcher,
indexed by ply, with a few sentinel entries before ply 0 so `ply - 2` never
underflows (use an offset).

**End state.** The stack in place and killers moved into it; behaviour
unchanged (bench fingerprint identical to 3.3), so no game test.

### 3.5 Null move pruning

**What.** If the side to move could pass and still be above beta after a
reduced search, the position is very likely above beta anyway: return early.
It is one of the largest single gains in alpha-beta search.

**How it is usually done.** Conditions: non-PV node, not in check, static
evaluation at least beta, the side to move has non-pawn material (zugzwang is
common in pawn endings), the previous move was not a null move. Reduction
typically `3 + depth / 3`, sometimes plus a term from how far the evaluation
exceeds beta. Do not return an unproven mate score.

```rust
if !N::PV && !in_check && depth >= 3 && static_eval >= beta
    && board.has_non_pawn_material(board.side_to_move())
    && !self.stack[ply - 1].null_move
{
    let reduction = 3 + depth / 3;
    board.make_null_move();
    let score = -self.negamax::<NonPv>(board, depth - reduction, ply + 1, -beta, -beta + 1);
    board.unmake_null_move();
    if score >= beta {
        return if score >= MATE_IN_MAX_PLY { beta } else { score };
    }
}
```

**End state.** Null move pruning with tests on zugzwang positions (it must stay
off in king-and-pawn endings); SPRT `[0, 10]`.

### 3.6 Reverse futility pruning and razoring

**What.** *Reverse futility pruning* (static null move): at shallow depth, if
the static evaluation exceeds beta by a depth-scaled margin, return it.
*Razoring*: at very shallow depth, if the evaluation is far below alpha, drop
straight into quiescence and return if it confirms.

```rust
if !N::PV && !in_check && depth <= 8
    && static_eval - 80 * (depth - improving as i32) >= beta
{
    return static_eval;
}
```

Every margin here is a seed for SPSA (3.12).

**End state.** Both prunings; SPRT `[0, 5]` for each, or one SPRT for both if
you register them as one change.

### 3.7 Late move reductions

**What.** Moves late in a well-ordered list rarely matter: search them at
reduced depth with a null window, and re-search at full depth only if they beat
alpha.

**How it is usually done.** A table of base reductions from
`ln(depth) × ln(move number)`, adjusted by node properties.

```rust
static LMR: std::sync::LazyLock<[[u8; 64]; 64]> = std::sync::LazyLock::new(|| {
    let mut table = [[0u8; 64]; 64];
    for depth in 1..64 {
        for moves in 1..64 {
            table[depth][moves] = (0.75 + (depth as f64).ln() * (moves as f64).ln() / 2.25) as u8;
        }
    }
    table
});

let mut score = -INFINITE;
if depth >= 3 && move_count > 1 + N::PV as i32 && is_quiet {
    let mut r = LMR[depth.min(63) as usize][move_count.min(63) as usize] as i32;
    r -= N::PV as i32;          // reduce PV nodes less
    r += !improving as i32;     // reduce more when not improving
    let reduced = (depth - 1 - r).clamp(1, depth - 1);
    score = -self.negamax::<NonPv>(board, reduced, ply + 1, -alpha - 1, -alpha);
    if score > alpha && reduced < depth - 1 {
        score = -self.negamax::<NonPv>(board, depth - 1, ply + 1, -alpha - 1, -alpha);
    }
} else if !N::PV || move_count > 1 {
    score = -self.negamax::<NonPv>(board, depth - 1, ply + 1, -alpha - 1, -alpha);
}
if N::PV && (move_count == 1 || score > alpha) {
    score = -self.negamax::<Pv>(board, depth - 1, ply + 1, -beta, -alpha);
}
```

This merges PVS (3.3) and LMR into one re-search ladder. Common adjustments to
add one by one later: less for killers, more or less by history score, more for
captures that lost SEE, less when in check or giving check.

**End state.** LMR with its table and adjustments behind named constants; SPRT
`[0, 10]`.

### 3.8 Move-loop pruning: late move pruning, futility, SEE

**What.** Skip moves before searching them.

- **Late move pruning.** At shallow depth, after a depth-dependent number of
  quiet moves, skip the remaining quiets. A common count:
  `(3 + depth²) / (2 − improving)`.
- **Futility pruning.** At shallow depth, if the static evaluation plus a
  margin cannot reach alpha, skip quiet moves.
- **SEE pruning.** At shallow depth, skip captures with SEE below
  `−k × depth` and quiets below `−k × depth²`.

All three require that a non-losing move has already been found
(`best_score > -MATE_IN_MAX_PLY`), so the engine never prunes its way into
reporting a false mate.

**End state.** The three prunings; SPRT `[0, 5]` for each or registered as one.

### 3.9 Quiescence refinements

**What.** Make quiescence cheaper without losing tactics.

**How.** Skip captures with negative SEE; *delta pruning*: skip a capture when
stand-pat plus the victim's value plus a margin is still below alpha (never for
promotions; this replaces 1.4.0's duplicated branches, W-16); probe the table
in quiescence and use its move and cutoffs; store results.

**End state.** Refined quiescence; tactical suite at fixed nodes not worse;
SPRT `[0, 5]`.

### 3.10 Extensions and depth adjustments: check extension, IIR, mate distance pruning

**What.**

- **Check extension.** Search one ply deeper when a move gives check (or when
  in check). Once universal; modern engines often drop it in favour of other
  extensions. Measure it: this is a classic case where the answer depends on
  the rest of your search.
- **Internal iterative reduction.** When a node that should have a table move
  has none, reduce its depth by one: without a move to try first the search is
  expensive and probably not important.
- **Mate distance pruning.** If a mate shorter than any mate this node could
  produce is already known, cut: `alpha = alpha.max(mated_in(ply))`,
  `beta = beta.min(-mated_in(ply + 1))`, return if `alpha >= beta`.

**End state.** Each measured; the check extension kept or removed by its own
SPRT, IIR by `[0, 5]`; mate distance pruning verified by mate tests (it rarely
shows in games).

### 3.11 Aspiration windows

**What.** From depth 4 or so, search the root with a window around the previous
iteration's score; widen on failure.

```rust
let mut delta = 25;
let (mut alpha, mut beta) = if depth >= 4 {
    (previous_score - delta, previous_score + delta)
} else {
    (-INFINITE, INFINITE)
};
loop {
    let score = self.root_search(board, depth, alpha, beta);
    if self.stopped {
        break;
    }
    if score <= alpha {
        beta = (alpha + beta) / 2;
        alpha = (score - delta).max(-INFINITE);
    } else if score >= beta {
        beta = (score + delta).min(INFINITE);
    } else {
        break;
    }
    delta += delta / 2;
}
```

**End state.** Aspiration windows; fail-high and fail-low lines not printed as
exact scores (or printed with `lowerbound`/`upperbound`); SPRT `[0, 5]`.

### 3.12 Tunable parameters and the SPSA pipeline

**What.** Every margin, reduction constant and threshold chosen by hand in
Phase 3 is a guess. SPSA (simultaneous perturbation stochastic approximation)
fits many constants at once from game results: each iteration plays a small
match between two copies whose parameters are nudged in opposite random
directions, and moves the parameters toward the better side.

**Why only the pipeline now.** The centipawn margins are tied to the
evaluation's scale, and Phase 4 replaces the evaluation completely. A full
search tune now would fit margins to an evaluation that is about to disappear,
the same waste as Texel-fitting it. This step builds and proves the machinery;
the first full search SPSA is 5.7, on the finished evaluation.

#### 3.12.1 Tunables macro and the tune build

**How.** Declare every tunable once; a declarative macro generates the struct,
its defaults, the UCI `option` lines and the `setoption` handler. Expose the
options only in a `tune` Cargo feature so release builds do not advertise them.

```rust
macro_rules! tunables {
    ($($name:ident: $default:expr, $min:expr, $max:expr, $step:expr;)*) => {
        pub struct Tunables {
            $(pub $name: i32,)*
        }

        impl Default for Tunables {
            fn default() -> Self {
                Tunables { $($name: $default,)* }
            }
        }

        impl Tunables {
            pub fn uci_options() -> Vec<String> {
                vec![$(format!(
                    "option name {} type spin default {} min {} max {}",
                    stringify!($name), $default, $min, $max
                ),)*]
            }

            pub fn set(&mut self, name: &str, value: i32) -> bool {
                match name {
                    $(stringify!($name) => { self.$name = value; true })*
                    _ => false,
                }
            }

            /// The weather-factory `config.json` for the current defaults.
            pub fn weather_factory_config() -> String {
                let entries = vec![$(format!(
                    "  \"{}\": {{ \"value\": {}, \"min_value\": {}, \"max_value\": {}, \"step\": {} }}",
                    stringify!($name), $default, $min, $max, $step
                ),)*];
                format!("{{\n{}\n}}", entries.join(",\n"))
            }
        }
    };
}

tunables! {
    rfp_margin: 80, 20, 200, 10;
    nmp_base_reduction: 3, 1, 6, 1;
    lmr_divisor_x100: 225, 150, 350, 15;
}
```

**Rust you will practise.** `macro_rules!` repetitions, `stringify!`, macros in
pattern position, Cargo features and `#[cfg(feature = "tune")]`.

**End state.** A tune build whose `uci` lists every tunable and whose
`spsa` debug command prints the weather-factory configuration; the normal
build's bench fingerprint unchanged.

#### 3.12.2 weather-factory setup

**How.** weather-factory (`D:/chess/weather-factory`, a Python SPSA driver that
calls fastchess) reads three files in its folder. `config.json` is printed by
3.12.1. `cutechess.json` names the engine, book and match settings (`tc` is in
seconds and the increment is `tc/100`, so `3` means 3+0.03, the same control as
the SPRTs). `spsa.json` holds the SPSA constants; set `A` to about a tenth of
the planned iterations. Read the driver's source once to see exactly which
options it passes to fastchess: it should pass no adjudication options (this
plan's policy) and you should know whether it pins affinity, because the
tuning games then run under slightly different conditions from the SPRTs.

```json
{
  "engine": "whitespine-tune.exe",
  "book": "UHO_Lichess_4852_v1.epd",
  "games": 32,
  "threads": 14,
  "tc": 3,
  "hash": 16,
  "use_fastchess": true,
  "pgnout": "file=tuner/games.pgn",
  "save_rate": 10
}
```

Start command (from the weather-factory folder, maintainer-run):

```powershell
cd D:\chess\weather-factory
python main.py
```

**End state.** A tune folder with the three files, the tune binary and the
book.

#### 3.12.3 Pilot run that proves the pipeline

**How.** Run about 128 iterations over a handful of Phase 3 parameters. Prove
the wire is live first: set one parameter's starting value absurdly (for
example a reverse-futility margin of 1,000) and check that it moves back and
that the engine's node count changes with it. In the pilot, a parameter that
does not move may be inactive (check that the code reads it) or flat; one that
runs to its bound may need a wider range. Pilot values are never adopted.

**End state.** A pilot recorded in `EXPERIMENTS.md` with the parameter
trajectories and the proof that the options reach the engine; no values
committed.

### 3.13 Checkpoint and release 2.1.0

**What.** A pause to measure where the search stands.

**How.** Tier-2 gauntlet (re-placing engines between tiers as results
require); depth reached and branching factor on the bench; tactical suite at
fixed nodes; time-loss count in the SPRTs of this phase; calibration notes
for every Phase 3 prediction.

**End state.** Release 2.1.0 with a checkpoint entry in `EXPERIMENTS.md`.

---

## Phase 4 — The classical evaluation

**What this phase builds.** A complete, mature hand-crafted evaluation of the
kind the strongest classical engines converged on: material and piece-square
tables, material imbalance, pawn structure, mobility, piece placement, king
safety, threats, passed pawns, space, initiative, scale factors and an endgame
library with a KPK bitbase. That design is well documented in the chess
programming literature, and its families are described one by one below. It
will be replaced by a network in Phase 9; the point is to understand how a
strong evaluation is built, what it knows and why each part exists.

**How a classical evaluation is created.** The same loop for every family:

1. **Understand the chess idea** and the exact definition: which squares, which
   pieces, what counts as "safe", "weak" or "passed".
2. **Build the inputs once and share them.** Most families read the same
   attack maps, pawn information and king zone. Compute them in a fixed order
   (4.3.1) so each family can reuse what earlier ones produced.
3. **Seed the values.** A seed is a starting value on Whitespine's scale
   (4.1.1), chosen so the term has the right sign and a plausible size; it is
   not a result. Seed from chess judgement and from the published values in
   the literature, and keep the seeds of one family consistent with each other,
   because a family whose terms are on different scales fights itself before
   the tuner ever sees it.
4. **Trace every parameter** so the tuner in Phase 5 can fit it.
5. **Test**: activation on hand-picked positions (the term fires where it
   should and not elsewhere), colour symmetry, trace reconstruction.
6. **Gate** the family with an SPRT against the previous head.

**Why no Texel tuning in this phase.** Fitting a half-built evaluation spends
the fit on terms that will change meaning as soon as their neighbours arrive
(mobility changes what the piece-square tables should hold; king safety
changes what mobility should hold). The whole surface is fitted once, in Phase
5, when it is complete.

**Families that do not pass with seeds.** A seeded family may fail its SPRT
because its seeds are the wrong size for this search, not because the idea is
wrong. If its activation, symmetry and reconstruction tests are clean, do not
delete it: mark it **pending**, disable it with one constant, record it in
`EXPERIMENTS.md`, and move on. Phase 5 fits it with everything else and
decides it (5.5). A family with a failing test is a bug, not a pending family.

**How to work on a family.** Read the description here and the Chess
Programming Wiki page for the idea; write your version; test it. If you then
want to compare with an open-source implementation, do it after yours exists,
and treat any difference as a question ("why did they do that?") rather than
as a correction.

**Rust you will practise.** Organising a large module tree, `const` tables and
masks, small caches indexed by hash keys, `enum` dispatch instead of virtual
functions, generic tracing without runtime cost, bit manipulation in depth.

**End state of the phase.** Every family below implemented, traced, tested and
gated (accepted or pending), and a checkpoint (4.15) of the complete seeded
evaluation.

### 4.1 Evaluation architecture

#### 4.1.1 Value scale, piece values and seeds

**What.** Decide the unit of every evaluation number, and how seeds from any
source enter that unit.

**How.** Whitespine's search margins (Phase 3) are already in centipawns, so
keep centipawns internally, defined as **an endgame pawn = 100**. The
definition matters: piece values differ between middlegame and endgame, and a
scale needs one fixed point. Everything the search compares against the
evaluation (futility margins, aspiration deltas, the mate range) is then in the
same unit.

Seed the material from the classical values every chess player knows, with
the middlegame pawn a little below 100 (a pawn matters less while pieces
dominate) and pieces a little more valuable in the endgame:

| Piece | Middlegame | Endgame |
|---|---|---|
| Pawn | 80–90 | 100 |
| Knight | 320 | 330 |
| Bishop | 330 | 350 |
| Rook | 500 | 540 |
| Queen | 950 | 1,000 |

These are seeds; Phase 5 refits them, and the refit usually moves the minor
pieces and the rook noticeably. If you take a seed for any term from a
published source that uses another unit, convert it through **one** function
in **one** place and never write a converted number inline; a scale mixed by
hand is the hardest evaluation bug to find. Very small terms rounding to a few
centipawns is acceptable for seeds.

**End state.** The material constants, the unit stated in a comment at the
top of the evaluation module, and a single conversion function if any seed
needed one.

#### 4.1.2 Tapered score type and game phase

**What.** Most terms have different values in the middlegame and the endgame.
Keep two values per term, add them as one, and interpolate by a **phase**.

**How it is usually done.** A packed score holds both halves in one integer:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct S(i32);

impl S {
    pub const fn new(mg: i16, eg: i16) -> S {
        S(((eg as i32) << 16) + mg as i32)
    }
    pub const fn mg(self) -> i16 {
        self.0 as i16
    }
    pub const fn eg(self) -> i16 {
        ((self.0 + 0x8000) >> 16) as i16
    }
}

impl std::ops::Add for S {
    type Output = S;
    fn add(self, other: S) -> S {
        S(self.0 + other.0)
    }
}
```

The `+ 0x8000` in `eg()` corrects for the borrow a negative middlegame half
takes from the upper half; test negative halves before trusting it.

Make the phase continuous rather than a few discrete stages: total non-pawn
material, clamped between an endgame limit and a middlegame limit, mapped
linearly to 0–128. Seed the middlegame limit near the non-pawn material of a
full board minus a minor piece (about 6,000 cp on the values above) and the
endgame limit near a rook plus a minor piece (about 1,500 cp); between them
the evaluation slides smoothly from one set of values to the other, so a
trade never causes a jump. The endgame half is also multiplied by a **scale
factor** (4.12) out of 64 before interpolation:

```rust
let npm = board.non_pawn_material().clamp(ENDGAME_LIMIT, MIDGAME_LIMIT);
let phase = (npm - ENDGAME_LIMIT) * 128 / (MIDGAME_LIMIT - ENDGAME_LIMIT);
let value = (score.mg() as i32 * phase
    + score.eg() as i32 * (128 - phase) * scale_factor / 64) / 128;
```

**Rust you will practise.** Operator traits on your own types (`Add`, `Sub`,
`Neg`, `Mul<i32>`, `AddAssign`, `SubAssign`), two's complement reasoning.

**End state.** `S`, phase and interpolation with tests, including negative
halves and the scale factor.

#### 4.1.3 Parameters as data and the coefficient trace

**What.** A Texel tuner needs, for every position, how many times each
parameter was used for each side (its *coefficients*). Build that ability now,
while each term is written, instead of retrofitting it.

**How it is usually done.** Every constant lives in one parameter structure,
never inline. The evaluation is generic over a *tracer*: the engine uses one
whose methods are empty and vanish after inlining; the tools use one that
counts.

```rust
pub trait Tracer {
    fn add(&mut self, param: usize, color: Color, count: i32);
}

pub struct NoTrace;
impl Tracer for NoTrace {
    #[inline(always)]
    fn add(&mut self, _param: usize, _color: Color, _count: i32) {}
}

pub struct CoefficientTrace {
    pub counts: Vec<[i32; 2]>, // per parameter, per colour
}
impl Tracer for CoefficientTrace {
    fn add(&mut self, param: usize, color: Color, count: i32) {
        self.counts[param][color.index()] += count;
    }
}

// inside a family:
score += params.rook_on_file[open as usize];
trace.add(Params::ROOK_ON_FILE + open as usize, us, 1);
```

Some terms are not "count × parameter": king danger squares a sum, initiative
caps by sign, scale factors multiply. Trace their **inputs** too (for king
danger, the count of each danger unit), so Phase 5 can handle them (5.2.4).

**Rust you will practise.** Generic functions over traits, monomorphisation,
`#[inline(always)]`, associated constants as parameter indices.

**End state.** A parameter structure with named index constants, both tracers,
and the evaluation generic over the tracer.

#### 4.1.4 Symmetry and reconstruction tests

**What.** Two tests that catch most evaluation bugs, run after every family.

- **Colour symmetry.** Flip the position vertically and swap colours: the
  evaluation from the side to move's point of view must not change. A failure
  points at a term that treats the colours differently.
- **Reconstruction.** Rebuild the evaluation from the trace and the parameters
  (tapered, scaled, with the nonlinear parts recomputed from their traced
  inputs). It must equal the real evaluation exactly, or the tuner would fit a
  different function from the one the engine plays.

**End state.** Both tests over thousands of positions from the bench set,
perft suite and random games.

#### 4.1.5 The `eval` breakdown command

**What.** An `eval` command that prints a table: one row per family, White's
and Black's middlegame and endgame contribution, and the total. It is how you
debug and understand the evaluation: set up a position, predict which families
matter, compare with the table.

```text
      Term    |    White    |    Black    |    Total
              |   MG    EG  |   MG    EG  |   MG    EG
 -------------+-------------+-------------+-------------
     Material |  ----  ---- |  ----  ---- |  0.12  0.25
       Pawns  |  0.30  0.10 |  0.22  0.18 |  0.08 -0.08
 ...
```

**End state.** `eval` prints the table for every family that exists, growing
with each step of this phase.

### 4.2 Material and piece-square tables

**What.** The base of every classical evaluation: material, and for each piece
type a table of 64 tapered values saying where the piece likes to stand.

#### 4.2.1 Incrementally updated material and piece-square score

**How it is usually done.** Keep one packed score in the position:
`material + table value` for every piece, added in `add_piece`, subtracted in
`remove_piece`, both in `move_piece`. The evaluation reads it instead of
looping over pieces, which makes the cheapest part of the evaluation free at
every node. Keep the non-pawn material per colour the same way; the phase and
many families need it.

**End state.** `Board::psq_score()` and `non_pawn_material(color)` maintained
by the piece primitives and checked in `validate()` (1.3.3).

#### 4.2.2 Table shape and seeds

**How.** Store piece tables for files a–d only and mirror them to e–h (piece
placement is assumed left-right symmetric, which halves the parameter count
the tuner must fit), and pawn tables for all eight files (pawn placement is
not symmetric: castled kings and flank majorities differ). Black's values are
White's mirrored vertically with the sign flipped, so one table serves both
colours and symmetry is guaranteed by construction.

Seed the shapes from chess knowledge, tapered: knights and bishops toward the
centre and away from the rim; rooks to the seventh rank and open files in the
middlegame, less in the endgame; the queen slightly toward the centre; the
king to the castled corners in the middlegame and toward the centre in the
endgame; pawns rewarded for advancing far more in the endgame than in the
middlegame, with central pawns preferred on the fourth rank and edge pawns
neutral. Seed magnitudes of a few tens of centipawns; the tuner will reshape
every table.

**End state.** The 1.4.0 heuristic deleted; material and tables seeded, traced,
symmetric; SPRT `[0, 10]` against the Phase 3 head (the evaluation half of the
engine is replaced here). A **tempo bonus** for the side to move (seed 10–15
cp: having the move is worth something, and it keeps odd and even depths
from disagreeing) is added in the same step.

### 4.3 Evaluation pipeline and caches

#### 4.3.1 Evaluation order and the lazy exit

**What.** The order in which the evaluation runs, which is also the order in
which this phase builds the families, because later families read what earlier
ones computed.

```text
evaluate(position)                     [never called when in check]
  material entry  <- material hash (4.3.2)
    if a specialised endgame evaluation exists: return it (4.14)
  score  = material + tables (4.2) + imbalance (4.10)
  pawn entry      <- pawn hash (4.3.3)
  score += pawn structure (4.4)
  lazy exit: if |(mg + eg) / 2| is far beyond a threshold, return it
  initialise attack maps, king rings, mobility areas (4.3.4)
  score += pieces for N, B, R, Q, filling attack maps (4.5)
  score += mobility (4.5.1)
  score += king safety (4.6)   [reads all attack maps and mobility]
  score += threats (4.7)       [reads attack maps]
  score += passed pawns (4.8)  [reads attack maps and pawn entry]
  score += space (4.9)
  score += initiative (4.11)   [reads the score so far]
  scale factor (4.12), taper (4.1.2), side-to-move sign, tempo
```

**The lazy exit.** When material and pawns alone put the score far outside
any interesting range (a threshold of several pawns, growing a little with the
material on the board because more pieces mean the remaining families can
swing further), the rest of the evaluation cannot change the outcome of the
search, so it returns early and saves time. It changes behaviour slightly, and
it must be disabled while tracing for the tuner.

**End state.** The evaluation function has this skeleton with empty family
functions, and the lazy exit sits behind a constant that is off until 4.15.

#### 4.3.2 Material hash table

**What.** Everything that depends only on the piece counts is computed once
per material configuration and cached by the material key (1.4.3): the game
phase, the imbalance (4.10), the endgame evaluation or scaling function for
this material (4.14) and the default scale factors (4.12).

```rust
pub struct MaterialEntry {
    key: u64,
    imbalance: S,
    phase: u8,
    scale_factor: [u8; 2],                 // default factor per strong side
    evaluation: Option<EndgameEval>,       // a specialised evaluation, e.g. KBNK
    scaling: [Option<EndgameScale>; 2],    // a scaling function per strong side
}
```

**End state.** A per-thread material table (a few thousand entries) with a
hit-rate counter reported by `bench`.

#### 4.3.3 Pawn hash table

**What.** Pawn structure changes rarely between nodes, so everything computed
from pawns alone is cached by the pawn key: the pawn structure score per
colour (4.4), passed pawns, pawn attacks, the *pawn attack span* (squares
our pawns could ever attack; outposts need it), and the king shelter score
(4.6.1), which also depends on the king square and castling rights, so its
cache slot stores those and is recomputed when they change.

**End state.** A per-thread pawn table (tens of thousands of entries), hit rate
reported by `bench`.

#### 4.3.4 Attack maps, king ring, mobility area and blockers

**What.** The shared working state of one evaluation.

- `attacked_by[color][piece type]` and `attacked_by[color][ALL]`: squares
  attacked by each piece type.
- `attacked_by_2[color]`: squares attacked at least twice (x-rays included),
  the basis of "safe" and "strongly protected".
- `king_ring[color]`: the king's square and neighbours, with the king square
  moved away from the edge first (so a king on g1 gets a ring as if on g2),
  minus squares defended by two own pawns. The enemy's king-attacker count
  starts at the number of ring squares enemy pawns attack.
- `mobility_area[color]`: squares that count for mobility: not own pawns that
  are blocked or on the second or third rank, not own king or queen, not
  **blockers for the own king** (of either colour), not attacked by enemy
  pawns.
- **Blockers for the king**: pieces of **either** colour that stand alone
  between the king and an enemy slider. Own blockers are pinned; enemy
  blockers can give discovered check. Extend 1.5.2 to compute blockers and the
  pinning sliders for both kings in `make`.

```rust
pub struct EvalState {
    attacked_by: [[Bitboard; 7]; 2], // six piece types plus ALL
    attacked_by_2: [Bitboard; 2],
    king_ring: [Bitboard; 2],
    king_attackers_count: [i32; 2],
    king_attackers_weight: [i32; 2],
    king_attacks_count: [i32; 2],
    mobility_area: [Bitboard; 2],
    mobility: [S; 2],
}
```

**End state.** `EvalState` initialised for both colours from king and pawn
attacks before the piece loop, with tests for ring and area on edge kings and
pinned pieces.

### 4.4 Pawn structure

**What.** The pawn skeleton decides plans, weaknesses and endgames. Score
every pawn once (cached in the pawn table) from a handful of relations to the
pawns around it.

**How it is usually done.** For each pawn on square `s` with relative rank `r`:

| Relation | Definition |
|---|---|
| opposed | an enemy pawn is ahead on the same file |
| blocked | an enemy pawn is directly in front |
| stoppers | enemy pawns in the passed-pawn span (ahead on the same and adjacent files) |
| lever | enemy pawns this pawn attacks now |
| lever push | enemy pawns this pawn would attack after one push |
| doubled | an own pawn directly behind |
| neighbours | own pawns on adjacent files |
| phalanx | neighbours on the same rank |
| support | neighbours one rank behind (defending this pawn) |

Derived properties:

- **Backward**: no own neighbour level with or behind it, and it cannot
  advance safely (a lever push or a blocker).
- **Passed**: all stoppers are levers (it can capture its way through); or the
  only stoppers are lever-push pawns and the phalanx outnumbers them; or the
  only stopper is the blocker, the pawn is on rank 5 or beyond, and a
  supporting pawn can advance to challenge it.

Scores:

- **Connected** (supported or in a phalanx): a bonus growing with rank,
  multiplied by `2 + phalanx − opposed`, plus a bonus per supporter; the
  endgame half is the middlegame half times `(r − 2) / 4`.
- **Isolated** (no neighbours) or **backward**: a penalty, larger when the pawn
  is unopposed (a weakness on a half-open file the enemy rooks can use).
- **Doubled** and **weak lever** (attacked by two enemy pawns): penalties when
  the pawn has no support.

```rust
let support = neighbours & RANK_BB[(s - up).rank()];
let phalanx = neighbours & RANK_BB[s.rank()];
if (support | phalanx).any() {
    let v = connected[r] * (2 + phalanx.any() as i32 - opposed.any() as i32)
        + params.supported * support.count() as i32;
    score += S::new(v as i16, (v * (r as i32 - 2) / 4) as i16);
} else if neighbours.is_empty() {
    score -= params.isolated + params.weak_unopposed * !opposed.any() as i32;
}
```

The passed pawns found here are only **flagged**; their value depends on
pieces and kings and is computed in 4.8.

**End state.** Pawn structure in the pawn table, with activation tests for
every relation (the classic positions: an isolated d-pawn, a backward pawn on
a half-open file, a phalanx, a protected passer, the three passed-pawn
conditions); SPRT `[0, 5]`.

### 4.5 Pieces

**What.** One loop over knights, bishops, rooks and queens for each colour. It
fills the attack maps, counts attacks on the enemy king zone for 4.6, scores
mobility and scores placement.

**How the loop starts, for every piece.** Compute its attacks. Bishops see
through queens and rooks see through queens and own rooks (x-rays: a battery
controls squares behind the front piece). A piece that blocks an attack on its
own king may only move along the pin line. Update `attacked_by_2` (squares
already attacked), `attacked_by`, and, if the attacks touch the enemy king
ring, the attacker count, the attacker weight by piece type and the number of
attacked squares next to the enemy king.

```rust
let mut attacks = match kind {
    PieceType::Bishop => bishop_attacks(sq, occupied ^ queens),
    PieceType::Rook => rook_attacks(sq, occupied ^ queens ^ board.pieces(us, PieceType::Rook)),
    _ => piece_attacks(kind, sq, occupied),
};
if board.blockers_for_king(us).contains(sq) {
    attacks &= line(board.king_square(us), sq);
}
st.attacked_by_2[us.index()] |= st.attacked_by[us.index()][ALL] & attacks;
st.attacked_by[us.index()][kind.index()] |= attacks;
st.attacked_by[us.index()][ALL] |= attacks;
if (attacks & st.king_ring[them.index()]).any() {
    st.king_attackers_count[us.index()] += 1;
    st.king_attackers_weight[us.index()] += KING_ATTACK_WEIGHT[kind.index()];
    st.king_attacks_count[us.index()] += (attacks & st.attacked_by[them.index()][KING]).count() as i32;
}
```

#### 4.5.1 Mobility

**What.** A piece that attacks many useful squares is more valuable. The value
is a table per piece type indexed by the number of attacked squares inside the
mobility area (4.3.4): 9 entries for knights, 14 for bishops, 15 for rooks, 28
for queens. The tables are strongly nonlinear: a knight with 0–2 squares is
nearly trapped and heavily penalised, extra squares beyond 5 add little.

**End state.** Mobility tables seeded and traced (one parameter per entry);
activation tests (a trapped bishop, an open-board queen); SPRT `[0, 10]`
together with 4.5.2–4.5.3 or separately.

#### 4.5.2 Knights and bishops

**What.**

- **Outpost**: the minor stands on rank 4–6 (relative), is defended by an own
  pawn, and no enemy pawn can ever attack the square (outside the enemy pawn
  attack span from 4.3.3). Knights get double the bonus. A knight that can
  **reach** such a square next move gets a smaller bonus.
- **Minor behind pawn**: a minor directly behind a pawn (of either colour) is
  shielded and supports it.
- **King protector**: a penalty proportional to the distance from the own king
  (minors help defend when close).
- **Bishop pawns**: a penalty per own pawn on the bishop's square colour,
  multiplied by `1 + blocked own pawns on the centre files` (a bishop behind
  its own blocked centre is "bad").
- **Long diagonal bishop**: a bonus when the bishop sees both central squares
  of a long diagonal through pawns.

#### 4.5.3 Rooks and queens

**What.**

- **Rook on queen file**: a rook on the same file as any queen.
- **Rook on open or semi-open file**: no own pawn on the file gives the
  semi-open bonus; no pawn of either colour gives the open bonus.
- **Trapped rook**: a rook that is on neither an open nor a semi-open file,
  has at most 3 mobility, and stands between its own king and the nearer
  corner (king on files a–d and rook further toward the a-file, or the mirror
  on the king side); penalised twice as much when the side has no castling
  rights left.
- **Weak queen**: the queen can be pinned or hit by a discovered attack from an
  enemy rook or bishop (computed with the slider-blocker function for the queen
  square).

**End state of 4.5.** All piece terms traced with activation tests; SPRT.

### 4.6 King safety

**What.** The most valuable classical family and the most nonlinear: one
attacker near the king means little, three together can be decisive. Combine
a pawn-shelter score (cached per king square) with a **king danger** sum
converted by a square law.

#### 4.6.1 Pawn shelter and storm

**How it is usually done.** For the king's file and the two neighbouring files
(the king file clamped to b–g first), look only at pawns not behind the king:

- **Shelter**: the rank of the own pawn closest to the king on that file (0 if
  none) indexes a table by distance from the edge; high values for pawns on the
  second and third rank in front of the king, negative for missing pawns on the
  central files.
- **Storm**: the rank of the enemy pawn closest to the king indexes a second
  table; an advancing enemy pawn is dangerous. If that enemy pawn is directly
  blocked by the own shelter pawn and stands on the third rank, a separate
  smaller "blocked storm" penalty applies instead.
- **Castling alternative**: if the side can still castle, the shelter score
  after castling to either side is also computed and the best is used, so the
  engine is not punished for a king on e1 that will castle.
- **King–pawn distance** (endgame): a penalty per square of distance between
  the king and its nearest own pawn.

```rust
let centre = king_sq.file().clamp(1, 6);
let mut bonus = S::new(5, 5);
for file in centre - 1..=centre + 1 {
    let our_rank = rearmost_relative_rank(our_pawns & FILE_BB[file as usize], us);
    let their_rank = rearmost_relative_rank(their_pawns & FILE_BB[file as usize], us);
    let edge = file.min(7 - file) as usize;
    bonus += S::new(shelter[edge][our_rank], 0);
    if our_rank != 0 && our_rank + 1 == their_rank {
        bonus -= blocked_storm * (their_rank == 2) as i32;
    } else {
        bonus -= S::new(unblocked_storm[edge][their_rank], 0);
    }
}
```

(Ranks here are relative and zero-based, as in the tables' indexing; decide
your convention once and test it on both colours.)

#### 4.6.2 King danger

**How it is usually done.** Accumulate **danger units** for the king of side
`us`, from the enemy's point of view. Danger units are their own scale, not
centipawns: the sum is converted at the end. Each component is a count times
a weight parameter:

- **Attackers**: `attackers count × attackers weight`, both from the piece
  loop. The weight is per piece type; seed minors highest, the rook lower and
  the queen lowest, because a queen's danger is mostly counted through the
  safe-check terms below and would otherwise be counted twice.
- **Weak ring squares**: squares in the king ring attacked by the enemy, not
  defended twice by us, and either undefended or defended only by our king or
  queen (a defender that cannot afford to recapture).
- **Safe checks**: for each enemy piece type, whether it can give check next
  move from a square that is not attacked by us, or is attacked only weakly
  and doubly attacked by the enemy. One weight per piece type if any such
  square exists. Seed these as the heaviest units by far, several times an
  attacker's weight: a safe check is a concrete threat next move. Count queen
  checks only on squares where no rook check is possible and bishop checks only
  where no queen check is, so one square is not counted as three checks.
- **Unsafe checks**: checking squares that are defended; a small weight per
  square, because the check can still disrupt.
- **Blockers for our king**: pinned pieces and potential discovered checks.
- **Attacks next to the king**: one small weight per attacked adjacent square.
- **Flank attacks**: proportional to the *square* of the number of enemy
  attacks on our king flank within our camp (doubly attacked squares counted
  twice): a flank under sustained attack is worse than the sum of the attacks.
- **Mobility difference**: the middlegame mobility of the enemy minus ours; a
  side with more active pieces attacks better.
- **Reductions**: a large one when the enemy has no queen (most mating attacks
  need her), one for an own knight defending next to the king, a share of the
  middlegame shelter score, and a small one per own defended flank square; and
  a constant offset that sets where "no danger" sits.

Then convert, only when the total exceeds a threshold, with a **square law**
in the middlegame and a linear term in the endgame:

```rust
if danger > params.danger_threshold {
    score -= S::new(
        (danger * danger / params.danger_mg_divisor) as i16,
        (danger / params.danger_eg_divisor) as i16,
    );
}
```

The square law is the point: two moderate factors together cost far more than
twice one of them, which is how mating attacks work. The divisors set the
scale between danger units and centipawns; they and the weights are all seeds,
and the tuner fits the weights through the square (5.2.4).

#### 4.6.3 Flank terms

**What.** A penalty when the king's flank has no pawns at all (the king has no
shelter and enemy rooks have open files), and a small linear penalty per flank
attack (the same count as in 4.6.2).

**End state of 4.6.** King safety traced: shelter and storm table entries as
linear parameters, the danger sum's components as traced inputs (5.2.4);
activation tests (an open king under a queen and rook, a king behind an intact
shelter, a king facing a pawn storm); SPRT `[0, 10]`.

### 4.7 Threats

**What.** Attacks on enemy pieces the side to move can exploit or the opponent
must answer, all computed from the attack maps.

**Definitions.** *Strongly protected*: a square the enemy defends with a pawn,
or defends twice while we do not attack it twice. *Defended*: an enemy non-pawn
piece on a strongly protected square. *Weak*: an enemy piece not strongly
protected and attacked by us.

**Terms.**

- **Threat by minor**: for each defended or weak enemy piece attacked by our
  knights or bishops, a bonus by the attacked piece type.
- **Threat by rook**: for each weak enemy piece attacked by our rooks, a bonus
  by type.
- **Threat by king**: a weak enemy piece attacked by our king.
- **Hanging**: weak enemy pieces that are not defended at all, or non-pawn
  pieces we attack twice.
- **Restricted piece**: squares both sides attack where the enemy is not
  strongly protected (their pieces' moves are restricted).
- **Threat by safe pawn**: enemy non-pawn pieces attacked by our pawns that
  stand on safe squares.
- **Threat by pawn push**: enemy non-pawn pieces our pawns would attack after a
  safe single or double push.
- **Knight on queen** and **slider on queen**: squares from which our knights
  or sliders could attack the enemy queen next move, if the squares are safe
  (sliders need a double attack).

```rust
let strongly_protected = st.attacked_by[them][PAWN]
    | (st.attacked_by_2[them] & !st.attacked_by_2[us]);
let weak = board.color(them) & !strongly_protected & st.attacked_by[us][ALL];
let defended = board.color(them) & !board.pawns() & strongly_protected;
for sq in (defended | weak) & (st.attacked_by[us][KNIGHT] | st.attacked_by[us][BISHOP]) {
    score += params.threat_by_minor[board.piece_type_on(sq).index()];
}
```

**End state.** All threat terms traced with activation tests; SPRT `[0, 5]`.

### 4.8 Passed pawns

**What.** The pawn table flagged passed pawns (4.4); now they are valued with
full knowledge of pieces and kings. Passed pawns win endgames, and their value
grows steeply with rank.

**How it is usually done.** For each passed pawn of relative rank `r`:

- **Rank bonus** from a table (steeply growing: the seventh-rank bonus is many
  times the fourth).
- From rank 4 on, with a **rank weight** `w` that grows linearly with the rank
  (a far-advanced passer's surroundings matter far more than a distant one's),
  measured at the *block square* (the square in front):
  - **King proximity**: add the enemy king's distance to the block square
    (capped) times a parameter and subtract our king's distance times a
    smaller one, both scaled by `w`, endgame only; also subtract our king's
    distance to the square after that, so the king is drawn along the pawn's
    path.
  - **Free path**: if the block square is empty, look at the squares up to
    promotion and the passed-pawn span. Use one of three bonus levels: the
    largest when no square of the span is attacked, a middle one when only the
    path to promotion is safe, the smallest when only the block square is
    safe, else nothing. An enemy rook or queen behind the pawn makes every span
    square count as attacked (it x-rays through the pawn as it advances). Add a
    small extra when the block square is defended or an own rook or queen
    stands behind. Bonus `level × w` to both halves.
- **Candidate halving**: if the pawn would not be passed after one push, or a
  pawn stands directly in front, halve the bonus.
- **Passed file**: a small penalty growing toward the centre files (edge
  passers are harder to stop).

```rust
let block = s + up;
let mut bonus = params.passed_rank[r];
if r >= 3 {
    let w = params.passed_weight_slope * r as i32 - params.passed_weight_offset;
    let eg = (king_distance(them_king, block).min(5) * params.passed_their_king
        - king_distance(our_king, block).min(5) * params.passed_our_king) * w;
    bonus += S::new(0, eg as i16);
    // ... second push, free path level, defended block square ...
}
```

**End state.** Passed-pawn terms traced (the king-proximity coefficients are
linear in their factors: trace distance × w), activation tests (an unstoppable
pawn, a blockaded passer, a passer with the enemy king in front); SPRT
`[0, 5]`.

### 4.9 Space

**What.** In the opening and middlegame, controlling safe squares behind one's
pawn chain in the centre gives pieces room.

**How it is usually done.** Only while non-pawn material is high (a threshold
near the material of a full board, so the term switches off once trades
begin; space no longer matters when there are few pieces to use it). Count
safe squares in the centre files on the own second to fourth ranks (not own
pawns, not attacked by enemy pawns); count a safe square a second time when it
is up to three squares behind an own pawn and no enemy piece attacks it.
Multiply by the square of the own piece count (space is worth more the more
pieces can use it) and divide by a constant; middlegame only.

**End state.** Space traced (its parameter is the multiplier; trace the
computed count × weight² / 16); SPRT `[0, 5]`.

### 4.10 Material imbalance

**What.** Piece values are not additive. A bishop pair is worth more than two
bishops' material; knights gain with pawns on the board; rooks and minor pieces
trade differently depending on what else is left.

**How it is usually done.** Tord Romstad's second-degree polynomial: counts
per colour for (bishop pair as a pseudo-piece, pawn, knight, bishop, rook,
queen); for each own piece type `i` present, sum over types `j ≤ i`
`ours[i][j] × own count[j] + theirs[i][j] × enemy count[j]`, multiply by own
count[i]; do this for both colours, subtract, divide by 16. Cached in the
material table.

```rust
fn imbalance(us: &[i32; 6], them: &[i32; 6], ours: &[[i32; 6]; 6], theirs: &[[i32; 6]; 6]) -> i32 {
    let mut bonus = 0;
    for i in 0..6 {
        if us[i] == 0 {
            continue;
        }
        let mut v = 0;
        for j in 0..=i {
            v += ours[i][j] * us[j] + theirs[i][j] * them[j];
        }
        bonus += us[i] * v;
    }
    bonus
}
// imbalance = (imbalance(white, black) - imbalance(black, white)) / 16, applied to both halves
```

**End state.** Imbalance in the material table, traced (each coefficient's
count is `count[i] × count[j]`, divided by 16); SPRT `[0, 5]`.

### 4.11 Initiative

**What.** A correction of the endgame half (and mildly the middlegame half) by
how *winnable* the position is for the side that is ahead: many pawns, pawns on
both flanks, passed pawns and active kings make an advantage count; symmetrical
pawns on one flank make it drawish.

**How it is usually done.** Compute a *complexity* score as a weighted sum of
features that make a position winnable, minus a constant:

| Feature | Why it counts |
|---|---|
| number of passed pawns | a passer is a concrete way to win |
| number of pawns | more pawns, more play; pawnless positions are drawish |
| outflanking (king file distance minus king rank distance) | kings far apart on the files leave room for a decisive king march |
| infiltration (a king has crossed into the enemy half) | an active king in an endgame is a winning factor |
| pawns on both flanks | play on two wings stretches the defence |
| no non-pawn material | pure pawn endings are decided by tempo and are rarely drawn by fortress |
| almost unwinnable (no passers, negative outflanking, pawns on one flank only) | a large negative weight: the classic drawn structure |

Each weight is a parameter. Apply the complexity in the direction of the
current advantage, capped so it never flips the sign of either half: it can
say "this advantage is worth less" but never "the other side is better".

```rust
let sign = |x: i32| (x > 0) as i32 - (x < 0) as i32;
let u = sign(mg) * (complexity + params.initiative_mg_offset).min(0).max(-mg.abs());
let v = sign(eg) * complexity.max(-eg.abs());
score += S::new(u as i16, v as i16);
```

The middlegame half is only ever reduced, and mildly; the endgame half can be
raised or reduced, because winnability is an endgame concept.

**End state.** Initiative with its inputs traced (5.2.4 decides how it is
fitted); activation tests; SPRT `[0, 5]`.

### 4.12 Scale factors

**What.** A multiplier (0–64, 64 = normal) on the endgame half for the side
that is ahead, expressing "the material says winning, the position says
drawish".

**How it is usually done.**

- **From material** (material table): a side with no pawns whose non-pawn
  material advantage is at most a bishop cannot win by force: a factor near
  zero if it has less than a rook's worth of material, small if the other side
  has at most a bishop, moderate otherwise (a rook against a minor still has
  practical chances).
- **From specialised scaling functions** (4.14.5), when one exists for the
  material.
- **Generic** (only when nothing specific applied): opposite-coloured bishops
  as the only pieces give a low fixed factor (about a third); otherwise the
  factor is capped at `base + per_pawn × own pawns`, with a much smaller
  `per_pawn` when opposite bishops are present alongside other pieces, so few
  pawns mean less winning chance.
- **Fifty-move clock**: subtract a term growing with the halfmove clock, so a
  long shuffling phase slowly draws the evaluation toward zero and the search
  prefers lines that make progress.

**End state.** Scale factors in the evaluation, their inputs traced; tests on
opposite-bishop, pawnless and long-clock positions; SPRT `[0, 5]` on an
endgame-start book (a few thousand balanced endgame positions from your
games) plus an ordinary SPRT.

### 4.13 Syzygy tablebases through vendored Fathom

**What.** Syzygy tablebases give exact win/draw/loss (WDL) and distance to
zeroing (DTZ) for positions with few pieces. Fathom is a small C library that
probes them. It is vendored (its source copied into the repository, so a build
never depends on the network or on an upstream change) and compiled with the
`cc` build dependency; `cc` is the only dependency Whitespine will ever have.

**Why here.** The endgame library (4.14) is measured against tablebase truth,
and tablebases are a playing gain once installed.

#### 4.13.1 Vendoring and `build.rs` with `cc`

**How.** Copy Fathom's `src` folder (`tbprobe.c`, `tbprobe.h`, `tbchess.c`,
`tbconfig.h`, `stdendian.h`) and its `LICENSE` into `vendor/fathom/` from the
upstream repository, and note the upstream commit in a `VERSION` file next to
it. A build script compiles it before the crate.

```toml
[build-dependencies]
cc = "1"
```

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=vendor/fathom/src");
    let mut build = cc::Build::new();
    build
        .file("vendor/fathom/src/tbprobe.c")
        .include("vendor/fathom/src")
        .define("TB_NO_HELPER_API", None)
        .warnings(false);
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        build.cpp(true).flag_if_supported("/TP").flag_if_supported("/std:c++17");
    } else {
        build.flag_if_supported("-std=c11");
    }
    build.compile("fathom");
}
```

Two pitfalls worth a comment in `build.rs`: Fathom's C uses features that
MSVC's C mode rejects, so on MSVC it is compiled as C++ (the `/TP` flag above);
and a target CPU flag given to `rustc` does not reach the C compiler, so a
build meant for CPUs without POPCNT must define `TB_NO_HW_POP_COUNT` or
Fathom emits the instruction anyway and the binary dies with an illegal
instruction on such a CPU. The release matrix (8.4) has to set this per tier.

**Rust you will practise.** Build scripts, `cargo:` directives, build
dependencies, how Cargo links a static C library.

#### 4.13.2 FFI layer and UCI options

**How.** Declare the C functions in an `unsafe extern "C"` block, keep every
`unsafe` call inside one small module with a `// SAFETY:` comment saying why
it is sound, and expose safe Rust functions to the rest of the engine.

```rust
use std::ffi::{CString, c_char, c_uint};

unsafe extern "C" {
    static mut TB_LARGEST: c_uint;
    fn tb_init(path: *const c_char) -> bool;
    fn tb_probe_wdl_impl(
        white: u64, black: u64, kings: u64, queens: u64, rooks: u64,
        bishops: u64, knights: u64, pawns: u64, ep: c_uint, turn: bool,
    ) -> c_uint;
}

pub fn init(path: &str) -> Result<u32, TablebaseError> {
    let c_path = CString::new(path).map_err(|_| TablebaseError::NulInPath)?;
    // SAFETY: `c_path` is a valid NUL-terminated string that lives for the whole call.
    let ok = unsafe { tb_init(c_path.as_ptr()) };
    if !ok {
        return Err(TablebaseError::InitFailed);
    }
    // SAFETY: a by-value read of the global that `tb_init` has just written.
    Ok(unsafe { TB_LARGEST })
}
```

Read `tbprobe.h` for the exact contracts: which functions require the
fifty-move counter to be zero, that castling rights make a position
unprobeable, what `TB_RESULT_FAILED` is, how results and moves are packed, and
that `turn` is `true` for White. Options: `SyzygyPath` (string, empty
disables), `SyzygyProbeDepth` (spin), `Syzygy50MoveRule` (check). Report
`tbhits` in info lines.

**Rust you will practise.** `unsafe extern` blocks (edition 2024), raw
pointers, `CString`, `static mut` read by value, safe wrappers, `// SAFETY:`
discipline, tests that skip when no tablebases are installed.

#### 4.13.3 Search integration: interior WDL and root DTZ

**How it is usually done.**

- **Interior**: when the piece count is at most `TB_LARGEST`, there are no
  castling rights, the halfmove clock is zero (a capture or pawn move just
  happened) and the depth is at least `SyzygyProbeDepth`, probe WDL; convert a
  win to a score just below the mate range minus the ply, a loss symmetrically,
  a draw to 0; store in the table with an exact bound and return.
- **Root**: probe DTZ and keep only the root moves that preserve the best
  result, ranked so the engine makes progress under the fifty-move rule. Then
  search normally among them.

**End state.** Tablebase support tested on known positions (a KRK win found
fast, a KRKR draw held, a DTZ-optimal conversion in a KQK position), off by
default, on in the checkpoint gauntlets.

### 4.14 Endgames

**What.** An endgame library: specialised **evaluation functions** that
replace the whole evaluation for a material configuration (KBNK, KRKP, …), and
**scaling functions** that only set the scale factor (KRPKR, KBPsK, …). The
general evaluation knows nothing about the specific technique these endings
need (driving a king to the right corner, a fortress, a Philidor defence).
Rating lists often play without tablebases, and even with them the search must
first reach the probe positions, so this knowledge matters.

#### 4.14.1 Endgame dispatch by material key

**How it is usually done.** Build a map from material key to function at
start-up, computing each key from a position set up from a code string like
`"KRPKR"`, once for each strong side; the material table then finds the
function by one lookup. In Rust, an `enum` of endgame kinds and a `match`
replace the function pointers or virtual dispatch of other languages:

```rust
#[derive(Clone, Copy)]
pub enum EndgameEval { Kxk, Kbnk, Kpk, Krkp, Krkb, Krkn, Kqkp, Kqkr, Knnk, Knnkp }

#[derive(Clone, Copy)]
pub struct Endgame<K> {
    kind: K,
    strong: Color,
}

impl Endgame<EndgameEval> {
    pub fn evaluate(self, board: &Board) -> i32 {
        match self.kind {
            EndgameEval::Kbnk => kbnk(board, self.strong),
            // ...
        }
    }
}
```

Generic cases that cover many material keys (KXK: any decisive material
against a lone king; KBPsK: bishop and pawns; KQKRPs; KPsK: pawns only;
KPKP) are detected by rules in the material table instead of by key.

**End state.** Material-table lookup returns the evaluation or scaling
function; `eval` names it.

#### 4.14.2 Mating evaluations: KXK, KBNK, KQKR, KNNKP

**What.**

- **KXK**: material plus a bonus for pushing the lone king to the edge and for
  bringing the kings together, and a "known win" offset when mate is forceable
  (a queen, a rook, bishop and knight, or two bishops of opposite colours).
  If the lone king is stalemated with the weak side to move, it is a draw.
- **KBNK**: the lone king must be driven to a corner of the bishop's colour:
  push-to-corner by the bishop's colour instead of any edge.
- **KQKR**: queen minus rook value plus edge and closeness bonuses.
- **KNNKP**: push the king to the corner and reward the pawn's advance (the
  win needs the pawn to exist).

**End state.** Scripted tests: KQK, KRK and KBNK mated from random starts
within the fifty-move rule at short fixed time without tablebases.

#### 4.14.3 KPK bitbase by retrograde analysis

**What.** King and pawn against king is exactly solvable in memory, and
generating it is a classic exercise.

**How it is usually done.** Index: white king (64) × black king (64) × pawn on
files a–d and ranks 2–7 (24, mirroring the other files) × side to move (2):
196,608 positions. Mark illegal positions; mark immediate results (the pawn
promotes safely: win; stalemate or the pawn is captured: draw). Then iterate:
a position with the pawn's side to move is a win if **some** move reaches a
win; with the defender to move it is a win if **every** move reaches a win.
Repeat until nothing changes; what remains is a draw. Store one bit per
position.

**End state.** A bitbase generated at start-up (milliseconds), used by the KPK
evaluation and by the KPKP scaling function; verified against Syzygy on every
KPK position.

#### 4.14.4 Drawish and won evaluations: KRKP, KRKB, KRKN, KQKP, KNNK

**What.**

- **KRKP**: a win unless the pawn is far advanced, supported by its king, and
  the attacking king is far away; the evaluation interpolates by king and pawn
  distances.
- **KRKB**: usually drawn; a small bonus for pushing the defending king to the
  edge.
- **KRKN**: slightly better winning chances, more when king and knight are far
  apart.
- **KQKP**: a win, except a pawn on the seventh rank on the a, c, f or h file
  supported by its king, where only king distance counts (a draw by stalemate
  tricks).
- **KNNK**: a draw (mate exists but cannot be forced).

#### 4.14.5 Scaling functions

**What.** Each returns a scale factor, or "none" when the pattern does not
apply, for its material signature.

| Function | Pattern it knows |
|---|---|
| KBPsK | rook pawns with a bishop that does not control the queening square and the defending king in front: draw; some blocked b/g-file patterns |
| KQKRPs | the rook on its third rank defended by a pawn, king behind: fortress |
| KRPKR | Philidor-type defences, the defending king in front of the pawn, rook pawn and knight-pawn special cases |
| KRPKB | rook pawns and bishop-controlled promotion paths |
| KRPPKRP | no passed pawn for the stronger side and an active defending king: drawish |
| KPsK | all pawns on one rook file blocked by the defending king: draw |
| KBPKB | defending king on the pawn's path on a square the bishop cannot attack, or opposite bishops: draw |
| KBPPKB | opposite bishops with two pawns: mostly drawn unless the pawns are far apart |
| KBPKN | defending king in front of the pawn on a square the knight's side cannot drive it from: draw |
| KNPK | rook pawn on the seventh with the defending king in the corner: draw |
| KNPKB | the bishop controls the pawn's path |
| KPKP | probes the KPK bitbase ignoring the weaker pawn when it cannot matter |

**End state.** The scaling functions with one test position per rule.

#### 4.14.6 Measurement against tablebases

**How.** For each material signature, sample positions (from your games or
random legal positions with that material) and compare the evaluation's
verdict (clearly winning, drawish, clearly losing) with the Syzygy WDL, with and
without the specialised function. Keep a function only if it reduces the
disagreement. Then an SPRT `[0, 5]` on an endgame-start book with tablebases
off.

**End state.** A disagreement table per function in `EXPERIMENTS.md` and the
gated library.

### 4.15 Complete evaluation checkpoint

**What.** A pause with the whole seeded evaluation in place.

**How.** Enable the lazy exit and gate it alone (`[-1.75, 0.25]`, it should
not lose); run symmetry and reconstruction over a million positions; measure
NPS cost per family (bench with families disabled one at a time, for the
record only); list pending families; SPRT of the complete evaluation head
against the Phase 3 head for the record; Tier-3 gauntlet; review `eval` tables
for positions from your own lost games.

**End state.** A checkpoint entry with the family list (accepted or pending),
the NPS costs and the gauntlet ratings. No release: the evaluation is not yet
fitted.

---

## Phase 5 — Tuning the classical evaluation

**Why now.** The evaluation is complete, so its parameters can be fitted
together, once, on data from the engine that uses them. Texel tuning fits the
linear part (the vast majority of parameters) offline in minutes to hours;
SPSA handles the nonlinear remainder with games; the search margins are then
refitted to the new evaluation scale.

**How the gate works.** Fit loss is a screen and a falsifier, never
acceptance. The loss measures how well a static number predicts results from
fixed positions; play depends on how the search uses that number at hundreds
of thousands of positions per move, most of them never in the corpus. A vector
with a better loss can and sometimes does lose Elo. Every fit is therefore
baked into the engine and decided by an SPRT; a fit whose loss did not improve
is not worth the games.

**Rust you will practise.** A tools crate in the workspace, large `Vec`s of
compact structs, `std::thread::scope` for data parallelism, buffered I/O,
floating-point accumulation, generating Rust source from a program.

**End state of the phase.** A fitted classical evaluation with its search
refitted, measurably stronger than the seeded evaluation, released as 2.2.0.

### 5.1 Training data

**What.** Positions labelled with the result of the game they came from. The
data decides what the tuner can learn; bad data fits a bad evaluation very
precisely.

#### 5.1.1 Self-play data generation

**How it is usually done.** The engine plays itself at a small fixed node count
(5,000 to 10,000 nodes per move is common), from positions made by 8–12 random
legal plies, discarding starts whose evaluation is already lopsided. Games are
played to the end with no adjudication, and every position is written with the
game result. Write this as a `datagen` tool in the `tools/` workspace, using
threads, one game per thread at a time.

Use a format that Phase 9 can reuse: one line per position,
`<FEN> | <score> | <result>`, where `result` is `1.0`, `0.5` or `0.0` from
White's point of view and `score` is the search score from White's point of
view. This is the common text format NNUE trainers read.

```text
rnbqkb1r/pp2pppp/3p1n2/8/3NP3/8/PPP2PPP/RNBQKB1R w KQkq - 1 5 | 34 | 0.5
```

**Rust you will practise.** A second crate that depends on the engine library,
`std::thread::scope`, `BufWriter`, a seeded PRNG per thread.

#### 5.1.2 Position filtering and labels

**How.** Keep only positions a static evaluation can judge: not in check, not
in the first few plies, and *quiet* (the quiescence search from the position
returns the static evaluation, or you replace the position by the leaf of its
quiescence principal variation). Label with the game result. Blending the
result with the search score (for example 0.75 result + 0.25 sigmoid of the
score) is a known variant; start with the pure result and record the choice.

#### 5.1.3 Splits and the corpus manifest

**How.** Split by **game**, not by position, into training, validation and
test sets (for example 90/5/5 by a hash of the game's first position), so no
game leaks between sets. Write a manifest: generator commit and binary hash,
node count, number of games and positions, result distribution, phase
distribution, material-signature distribution (so endgame families have
enough samples), file hashes. The test set is used once per fit, at the end.

**End state of 5.1.** A corpus of several million quiet positions with its
manifest, generated by the maintainer. Many hours of generation is normal; keep
the host free of matches while it runs.

### 5.2 Texel tuner

**What.** Find the parameter vector that minimises the difference between game
results and a sigmoid of the evaluation over millions of positions.

#### 5.2.1 Data loading and sparse coefficients

**How.** Load each position once, run the evaluation with `CoefficientTrace`
(lazy exit off), and store only non-zero coefficients as `(parameter index,
white count − black count)` pairs, plus the phase, the scale factor and the
traced inputs of the nonlinear terms. The tuner never touches a board again.

```rust
struct Sample {
    result: f32,             // 1.0, 0.5 or 0.0 from White's point of view
    phase: f32,              // 0.0 (endgame) to 1.0 (middlegame)
    scale: f32,              // scale factor / 64 applied to the endgame half
    coefficients: Box<[(u32, i16)]>,
    nonlinear: NonlinearInputs, // king danger units per side, initiative inputs
}
```

Positions handled by a specialised endgame evaluation (4.14) carry no
coefficients for the normal terms; exclude them from the fit.

#### 5.2.2 K fitting and the loss

**How it is usually done.** The loss is the mean squared error between result
and expected score:

```rust
fn expected_score(eval: f64, k: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(-k * eval / 400.0))
}
// loss = mean over samples of (result - expected_score(eval, k))^2
```

`K` maps centipawns to expected score. Fit it once with the parameters fixed,
by a one-dimensional search (golden section over 0.5–2.0), then keep it fixed;
otherwise `K` and the scale of every parameter trade against each other.

#### 5.2.3 Gradient optimiser, validation and export

**How it is usually done.** For linear parameters the gradient is exact: the
error times the sigmoid's slope times the coefficient, split between halves by
phase, with the endgame half multiplied by the scale. Adam converges well.

```rust
let e = expected_score(eval, k);
let slope = (e - sample.result as f64) * e * (1.0 - e); // constant factors go into the learning rate
for &(index, coefficient) in sample.coefficients.iter() {
    let c = coefficient as f64 * slope;
    gradient[index].mg += c * sample.phase as f64;
    gradient[index].eg += c * (1.0 - sample.phase as f64) * sample.scale as f64;
}
```

Compute gradients per chunk on scoped threads and sum. Evaluate validation loss
every epoch and keep the vector with the best validation loss. Export by
writing `src/eval/params.rs` with integer values; rebuild; confirm the engine's
evaluation of sample positions equals the tuner's rounded prediction.

#### 5.2.4 Nonlinear terms: king danger, initiative, scale factors

**What.** The three places where the evaluation is not a sum of count ×
parameter.

- **King danger**: the penalty is `danger² / D_mg` (middlegame) and
  `danger / D_eg` (endgame) with `danger = Σ weight × count`. The gradient with
  respect to a weight is `2 × danger × count / D_mg` in the middlegame half and
  `count / D_eg` in the endgame half, so the tuner can include the weights with
  the chain rule, recomputing `danger` per sample every epoch from the traced
  counts. The divisors and the threshold stay fixed in the fit.
- **Initiative**: capped by sign, so its gradient is zero where the cap binds
  and linear elsewhere; either apply the chain rule with that rule, or keep its
  weights fixed in the fit (its contribution carried as a fixed per-sample
  residual) and tune them by SPSA (5.6).
- **Scale factors**: a multiplier the tuner treats as a fixed per-sample value
  computed with the seed parameters; their own constants go to SPSA.

**End state of 5.2.** A tuner that fits from the seeded vector to a stable
optimum; a smoke test where one parameter is set to an absurd value and the fit
pulls it back; a test that the tuner's evaluation of a sample equals the
engine's.

### 5.3 Fitting manifest: free, fixed and excluded parameters

**What.** A file listing every parameter with a status: **free** (receives
gradient), **fixed** (structure: phase limits, thresholds, table sizes) or
**excluded** (nonlinear constants left to SPSA), each with a reason. The tuner
reads it. Without such a file, "the fit" is whatever the tuner happened to
touch, and two refits months apart cannot be compared; with it, every refit
says exactly what was and was not fitted, and adding a family to the
evaluation forces a decision about its status.

**End state.** The manifest, and a tuner check that the parameter count in the
manifest equals the parameter structure's.

### 5.4 First full fit and gate

**How.** Fit the whole free surface on the 5.1 corpus from the seeded vector,
pending families (4.15) enabled. Record K, settings, the training and
validation curves, the test loss once, the largest parameter moves, and the
vector's hash. Bake, rebuild, bench, SPRT `[0, 10]` against the 4.15 head.

**End state.** Fitted evaluation accepted and recorded, or rejected with the
fit's evidence.

### 5.5 Pending families after the fit

**What.** Decide each family that was pending in Phase 4.

**How.** With the fitted head as baseline, disable one pending family at a time
and run a removal SPRT `[-1.75, 0.25]`. If removing it does not lose, remove it
(record why it failed: redundant with another family, too sparse in the data,
or harmful). If removing it loses, it stays.

**End state.** No pending families left; each decision recorded.

### 5.6 SPSA of the nonlinear residue

**What.** The constants the linear fit cannot fit: king-danger conversion
thresholds, initiative weights, scale-factor constants, the lazy-exit
threshold, space threshold. One registered SPSA over those only, at 3+0.03
(3.12's pipeline), never mixed with search parameters.

**End state.** Tuned values baked and SPRT `[0, 3]`, or a written skip if a
pilot shows a flat surface.

### 5.7 Search margin re-fit on the new evaluation

**What.** Every centipawn margin in the search (reverse futility, razoring,
futility, SEE thresholds, null-move terms, aspiration delta) was chosen for
1.4.0's evaluation. One registered SPSA over those search constants with the
evaluation frozen, never mixed with evaluation parameters. This is also the
first full search SPSA; 3.12 only proved the pipeline.

**End state.** Tuned search constants, SPRT `[0, 5]`.

### 5.8 Candidate terms beyond the core set

**What.** With a fitted evaluation, measure whether terms outside the core
set of Phase 4 add anything. Candidates from the literature: **closedness**
(a measure of how blocked the pawn structure is, scaling knights up and
bishops and rooks down), **x-ray attacks** (slider attacks through one piece
counted as pressure), **rook behind a passed pawn** (the rook supports the
pawn's advance and stays active), **passer blockade** (a well-placed blocker,
especially a knight, neutralises a passer), **king-centre danger** (an
uncastled king with the queens on), **queen infiltration** (a queen deep in
the enemy camp on a square pawns cannot attack) and **outpost refinements**
(an outpost is worth more when it cannot be challenged by a minor piece).

**How.** One candidate at a time: look for positions where the fitted
evaluation misjudges (large error against the game result) and check whether
the candidate describes them; implement with trace; refit the new parameters
with the rest fixed, then briefly the whole surface; SPRT `[0, 3]`. Stop after
two consecutive rejections and record the remaining list for later.

**End state.** Accepted candidates in the evaluation; rejected ones recorded
with their evidence.

### 5.9 Refit cycles

**What.** Data generated by a weaker engine teaches positions the stronger
engine no longer reaches. Regenerate data with the current head, refit the
whole surface, and gate.

**How.** Each cycle: generate (5.1's pipeline, same manifest format), fit from
the current vector, SPRT `[0, 3]`. Stop at the first cycle that does not
accept. In the first cycle also fit from a neutral start and compare, to see
whether the seed history holds the fit back.

**End state.** Cycles run until one fails to accept; each recorded.

### 5.10 Checkpoint and release 2.2.0

**How.** Tier-3 gauntlet with tablebases off and on; conversion check (games
where Whitespine held at least a piece more for 12 plies and did not win);
symmetry and reconstruction on the final vector; calibration notes for every
Phase 4–5 prediction.

**End state.** Release 2.2.0 and a checkpoint entry.

---

## Phase 6 — Advanced search

**Why now, and why differently.** The evaluation is mature enough that search
refinements are judged on a realistic score scale. The mechanisms of this phase
are the ones that separate a modern alpha-beta search from a textbook one, and
they are coupled: histories feed move ordering, LMR and pruning; correction
histories change the static evaluation every margin reads; singular search
reuses the table and the stack. Adding them one at a time against a tuned neighbourhood often measures
nothing. From here on (working rule 11): implement a **dependency-complete
group**, test it for correctness, tune its live constants together by SPSA,
and gate the group with `[0, 3]` (or `[0, 5]` for a large group).

**A screen before games.** Keep a set of tactical positions (quiet-move mates
in particular) with the depth at which the current head solves them. After a
change, a position that needs more depth is investigated before any SPRT;
aggressive pruning often breaks exactly those positions.

**Rust you will practise.** Large tables on the heap without stack overflow,
references into tables held across calls (and why the borrow checker objects),
indices instead of references, `Box<[T; N]>` and zeroed allocation, careful
`i16`/`i32` conversions.

**End state of the phase.** The search described in "The target": clustered
table with ageing, the full history family, correction histories, LMR in
fractional units with history pruning, singular and multi-cut and negative
extensions, ProbCut, refined null move and quiescence, a root move list with
MultiPV; your own code and constants, at a Tier-3/Tier-4 checkpoint.

### 6.1 Transposition table 2.0: clusters, stored static eval, TT-PV, ageing

**What.** A denser, smarter table.

**How it is usually done.** Three 10-byte entries plus padding in a 32-byte
cluster, so two clusters share a 64-byte cache line. Each entry adds the raw
static evaluation (so a node with a table hit skips evaluation), a *TT-PV* bit
(this position was once on a principal variation; later used by LMR and
pruning) and a 5-bit age (the search generation), which replacement uses to
overwrite old entries before deep but stale ones.

```rust
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Entry {
    key: u16,
    mv: Move,
    score: i16,
    eval: i16,
    depth: u8,
    flags: u8, // bound: 2 bits, tt-pv: 1 bit, age: 5 bits
}

#[repr(C, align(32))]
struct Cluster {
    entries: [Entry; 3],
    _padding: [u8; 2],
}

const _: () = assert!(std::mem::size_of::<Cluster>() == 32);
```

Replacement within a cluster: take the matching key, else the entry with the
lowest `depth − 8 × age difference` (or a similar formula). Allocate the table
zeroed on the heap; resizing reallocates.

**End state.** The new table with tests for bucket selection, ageing and
replacement; static evaluation reused from the table; SPRT `[0, 3]`.

### 6.2 History family

**What.** More specific histories order moves better and give LMR and pruning
a reliable "how good is this move usually" signal.

#### 6.2.1 Capture history

**How.** `[moving piece][to square][captured type]`, updated like quiet history
on cutoffs and maluses. Captures are then ordered by MVV plus capture history
instead of MVV-LVA, and the good/bad split can use `see_ge(mv, -score / k)` so a
capture with a strong history survives a small SEE loss.

#### 6.2.2 Continuation histories

**How.** `[previous moved piece][previous to square][piece][to square]`: how
good a move is *as an answer to* the move one ply earlier, and separately two,
four (and sometimes six) plies earlier. The stack entry for each ply holds an
index (or pointer) to the sub-table selected by the move made there; the quiet
score is the sum of the main history and the continuation histories.

```rust
// one table: 12 × 64 sub-tables, each 12 × 64 entries of i16 (about 1.2 MB)
pub struct ContinuationHistory {
    table: Box<[[[i16; 64]; 12]]>, // len = 12 * 64
}

impl ContinuationHistory {
    pub fn new() -> Self {
        ContinuationHistory {
            table: vec![[[0i16; 64]; 12]; 12 * 64].into_boxed_slice(),
        }
    }
}
```

Building this as a fixed array on the stack (`Box::new([[[[0; 64]; 12]; 64]; 12])`)
would first build 1.2 MB on the stack; in debug builds that crashes. The `vec!`
form allocates directly on the heap.

#### 6.2.3 Threat-aware quiet history and pawn history

**How.** Index quiet history additionally by whether the from and to squares are
attacked by the opponent (escaping a threat and moving into one are different
moves); add a pawn-structure history indexed by a bucket of the pawn key. A
per-position `threats` bitboard (squares attacked by the opponent, by piece
class) becomes a board producer computed once per node.

**End state of 6.2.** The histories with unit tests for bounds and gravity, the
killer question decided by measurement (several strong engines drop killers
once continuation histories exist; gate a removal with `[-1.75, 0.25]`), and
one SPRT `[0, 5]` for the group after a short SPSA of the bonus and malus
formulas.

### 6.3 Correction histories and the corrected static eval

**What.** The static evaluation is systematically wrong in some structures
(it misjudges a pawn structure, a piece configuration). *Correction history*
learns, per pawn key bucket (and per non-pawn key per colour, and per recent
move pair), the average difference between search results and static
evaluation, and adds it to the static evaluation before pruning reads it.

**How it is usually done.** After a node's search, if the best move is quiet,
not in check, and the result's bound agrees with the direction of the error,
update each table entry toward `score − raw_eval` with a weight growing with
depth. The corrected evaluation is `raw_eval + Σ corrections / grain`, clamped
out of the mate range. Rule-50 damping is usually applied to the corrected
value.

```rust
let bonus = ((score - raw_eval) * depth / 8).clamp(-CORRECTION_LIMIT, CORRECTION_LIMIT);
let entry = &mut self.pawn_correction[us.index()][pawn_key as usize % PAWN_CORRECTION_SIZE];
let value = *entry as i32;
*entry = (value + bonus - value * bonus.abs() / CORRECTION_LIMIT) as i16; // gravity, as in histories
```

**End state.** Pawn, non-pawn and continuation corrections; RFP, razoring,
futility and null-move conditions read the corrected evaluation; SPRT `[0, 5]`
after a short SPSA of weights and limits.

### 6.4 LMR 2.0 and history pruning

**What.** Reductions that use everything now known about a move and a node.

**How it is usually done.** Compute the reduction in fractional units (for
example 1,024ths of a ply) from the base table plus terms: history score of the
move, improving, TT-PV, cut node, the node being a PV node, the correction's
magnitude (a large correction means the evaluation is unreliable here), the
move giving check, the number of cutoffs the child produced. After the reduced
search beats alpha, decide from the reduced score whether the full-depth
re-search should go one ply deeper or shallower. *History pruning*: at shallow
depth skip quiet moves whose history is below a depth-scaled threshold.

**End state.** LMR 2.0 and history pruning as one group, SPSA over its terms,
SPRT `[0, 3]`.

### 6.5 Singular extensions, multi-cut and negative extensions

**What.** If the table move is much better than every alternative, extend it:
it is the only move and deserves more depth.

**How it is usually done.** At sufficient depth, when the table entry has a
lower bound and enough depth, search all moves except the table move (the stack
holds the *excluded move*) at about half depth, with a window just below
`tt_score − margin × depth`. If all fail low, extend the table move by one (by
two or three when they fail low by a large margin, with a cap on total
extensions). If the reduced search already beats beta, return (*multi-cut*). If
the table score itself is at least beta, or the node is a cut node, reduce the
table move instead (*negative extension*).

**Traps.** The excluded-move search must not store to or cut from the table
under the same key semantics; a cap on double extensions prevents search
explosions.

**End state.** Singular extensions with multi-cut and negative extensions; the
tactical screen passed; SPRT `[0, 5]` after SPSA of the margins.

### 6.6 ProbCut and null move refinements

**What.** *ProbCut*: at sufficient depth, if a capture's shallow search beats
`beta + margin`, a full search very probably beats beta; confirm with
quiescence, then with a reduced search, and return. *Null move refinements*:
reduction that grows with `eval − beta`; a verification search at high depth
(null move pruning disabled for a few plies) against zugzwang; an entry margin
above beta.

**End state.** Both as one group; SPSA of margins; SPRT `[0, 3]`.

### 6.7 Quiescence 2.0

**What.** A cheaper, smarter quiescence: corrected stand-pat; table probe and
store on every exit; move-count pruning after a few captures when not in check;
SEE pruning by a margin from alpha; fail-high scores pulled toward beta.

**Trap.** Once pruning in the main search is aggressive, some mate threats by
quiet checks are only seen if quiescence generates quiet checks at its first
ply: the main search prunes the quiet move that would have found them, and a
captures-only quiescence never looks. Engines that removed quiet checks from
quiescence early have had to put them back at the first ply later. Decide it
with the tactical screen, not by assumption.

**End state.** Quiescence 2.0; screen passed; SPRT `[0, 3]`.

### 6.8 Root: root moves, MultiPV and `searchmoves`

**What.** A root move list that remembers, per move, its score, previous score,
nodes spent and PV. It supports:

- **MultiPV**: search the first *k* moves each with its own full window, report
  `info multipv i`, and keep them sorted. A GUI analysing positions uses this.
- **`searchmoves`**: restrict the root to the given moves.
- **Time management inputs** for 7.1: the fraction of nodes spent on the best
  move and how often the best move changed.
- **Stability**: keep the previous best move when a re-search is interrupted.

**How it is usually done.** For MultiPV index `i`, search the root with the
moves before `i` excluded, aspiration window around the move's previous score;
after each index, stable-sort the moves from `i` on by score.

**End state.** `MultiPV` option (1–64, default 1) and `searchmoves` working and
tested in a GUI; with MultiPV 1 the bench fingerprint equals the previous head
(the root list refactor is behaviour-neutral at MultiPV 1).

### 6.9 Search SPSA over clusters

**What.** One registered SPSA over the search constants that Phase 6 left live
and that measurably interact (for example LMR terms with pruning thresholds),
never mixed with evaluation parameters. Run it only if the per-group tunes and
a pilot show the constants are still moving; a flat pilot is a reason to skip.

**End state.** Tuned values or a written skip; SPRT `[0, 3]` if tuned.

### 6.10 Checkpoint and release 2.3.0

**How.** Tier-3 and Tier-4 gauntlets; bench depth and branching factor;
tactical screen; time-loss count; calibration notes for every Phase 6
prediction (sign, size, which interaction was missed).

**End state.** Release 2.3.0 and a checkpoint entry.

---

## Phase 7 — Clock, pondering, threads and robustness

**Why now.** Time management and threads multiply whatever the search is worth,
so they come after the search and evaluation are mature; tuning a clock around
a search that is still changing wastes the tune.

**End state of the phase.** An engine that uses its time well at any control,
ponders, scales to several threads, and survives long tournaments without a
single crash or time loss.

### 7.1 Time management v2

**What.** Spend time where the position needs it.

**How it is usually done.** Keep the soft and hard limits of 2.4, computed at
`go` from remaining time, increment and moves to go, and never extend the hard
limit. After each completed iteration, scale the soft limit by factors from the
root (6.8): **best-move stability** (unchanged for several iterations: stop
earlier), **node fraction** (most nodes spent on the best move: it is clear;
few: keep thinking), **score trend** (score dropping: think longer). Do not
stop before a minimum depth.

```rust
let stability = [2.2, 1.6, 1.3, 1.1, 1.0, 0.9][best_move_stable_iterations.min(5)];
let fraction = best_move_nodes as f64 / total_nodes as f64;
let effort = (1.5 - fraction) * 1.35;
let trend = (1.0 + (previous_score - score).clamp(0, 60) as f64 / 120.0).min(1.5);
let scaled_soft = (soft_ms as f64 * stability * effort * trend) as u64;
if elapsed_ms >= scaled_soft.min(hard_ms) {
    break; // do not start another iteration
}
```

All factors are seeds for SPSA at a short control, then a check at a longer
one.

**End state.** Zero time losses in 10,000 self-play games across 1+0.01, 3+0.03
and `40/60` controls; SPRT `[0, 3]` at 3+0.03 and a confirmation at 10+0.1.

### 7.2 Pondering

**What.** Think on the opponent's time about the expected reply.

**How it is usually done.** `bestmove <move> ponder <reply>` (the reply from the
PV or the table); `go ponder …` starts a search with no time limit but with the
limits computed; `ponderhit` turns it into a normal timed search, with the
budget counted from `ponderhit`; `stop` during pondering ends it and the GUI
sends a new `position`. Advertise `Ponder` (check).

**End state.** Pondering tested in a GUI and in fastchess with ponder enabled;
zero protocol errors over 1,000 games.

### 7.3 Lazy SMP

**What.** Several threads search the same position independently and share
only the transposition table (and a few counters). The table makes them help
each other; small differences between threads make them explore different
parts of the tree.

**Rust you will practise.** `Send` and `Sync`, `std::thread::scope`, atomics,
interior mutability, `unsafe impl Sync` with a written justification, measuring
thread scaling.

#### 7.3.1 Shared, thread-safe transposition table

**How it is usually done.** Threads read and write entries without locks. A
torn read (half an old entry, half a new one) must be harmless: verify the key,
validate the move with `is_pseudo_legal` before use, and clamp scores. In Rust
this means either atomics (`AtomicU64` words, with the key XORed with the data
so a torn entry fails verification) or an `UnsafeCell` table with an explicit
`unsafe impl Sync` and a comment explaining why races are tolerated.

#### 7.3.2 Helper threads and result selection

**How.** Each thread owns its board copy, stack, histories and node counter;
the stop flag and the table are shared. The main thread controls time and
prints output; when it stops, it sets the flag and picks the best result (its
own, or a helper's that reached a deeper completed depth with a better score).

```rust
std::thread::scope(|scope| {
    for id in 1..thread_count {
        let (table, stop, board) = (&table, &stop, board.clone());
        scope.spawn(move || Worker::new(id, table, stop).search(board));
    }
    let result = Worker::new(0, &table, &stop).search_and_report(board);
    stop.store(true, Ordering::Relaxed);
    result
});
```

#### 7.3.3 Scaling measurement

**How.** Measure each thread count against the engine itself at one thread
(1T, 2T, 4T, 8T, same time per move), without `-use-affinity` (fastchess 1.8.0
pins one core per game, which starves multi-thread engines). Compare the gain
with published thread-scaling curves if you want a reference point; lazy SMP
typically gains well under the ideal of one doubling per doubling of threads.

**End state of 7.3.** `Threads` up to the machine's core count; the scaling
curve recorded. Beating one thread is expected and proves little; the gate is a
4T SPRT `[0, 5]` of the new head against the previous head, both at four
threads and without affinity.

### 7.4 Protocol robustness and engine lifecycle

**What.** Everything a GUI or a tournament manager can do to an engine.

**How.** A scripted stress test: random interleavings of `ucinewgame`,
`position`, `go` with every limit, `stop`, `isready`, `setoption` during and
between searches, `quit` during a search, input closed without `quit`, very
long move lists (games of 500+ plies), `go` with zero time left, `Hash` resized
during a game. Decide what a panic does (with `panic = "abort"` the process
ends; print the position to stderr first with a panic hook). No `unwrap` or
`expect` on anything that comes from input.

**End state.** The stress script runs thousands of iterations clean; a Tier-3
tournament of at least 5,000 games shows zero crashes, zero illegal moves and
zero time losses.

### 7.5 Checkpoint: four threads and long time control

**How.** Tier-4 gauntlet at 1T and at 4T; a self-play run at 10+0.1 between
the Phase 6 head and this head; time losses over the whole phase.

**End state.** A checkpoint entry; release when you choose (2.4.0).

---

## Phase 8 — Performance, release engineering and the classical checkpoint

**Why last before NNUE.** Speed work pays when the code it speeds up has
stopped changing, and the release pipeline is worth building once there is a
strong engine to ship. Every speed change must reproduce the bench fingerprint
exactly; NPS gains are measured, not assumed.

**How to measure speed.** Build each arm several times (two identical builds
differ by a few tenths of a percent), run bench interleaved (A, B, A, B, …) on
an idle host, compare medians, and first run the method on two copies of the
same binary: it must read about 0%. Speed is not Elo; convert only through a
game test.

**End state of the phase.** The fastest version of the classical engine,
released as per-CPU-tier PGO assets from a CI that asserts one fingerprint, and a
measured standing against the ladder's top tier.

### 8.1 Profiling workflow

**What.** Find where time goes before optimising.

**How.** Build with the `profiling` profile (release with debug symbols, 0.1).
On Windows use a sampling profiler that reads PDB symbols (for example Intel
VTune, Windows Performance Analyzer, Superluminal, or `samply`); on Linux,
`perf`. Profile `bench` and a real search from a middlegame position. Look for
allocations in the search (there should be none after start-up), evaluation
share, move generation share, table misses.

**End state.** A profile summary (share of time per function family) recorded
as an observation.

### 8.2 Throughput work: caches, prefetch, layout

**What.** The usual wins, each measured alone.

- **Prefetch** the table cluster for the child position before making the move
  (`core::arch::x86_64::_mm_prefetch`), and the pawn table entry.
- An **evaluation cache** keyed by the position hash.
- **Layout**: keep hot fields of the board and search state together; check
  `size_of` of hot structures.
- **Bounds checks**: remove them in hot loops by construction (iterators,
  fixed-size arrays indexed by types that cannot exceed the length) before
  reaching for `get_unchecked`, which needs a proven invariant and a
  `// SAFETY:` comment.
- **Inlining**: `#[inline]` on small cross-module functions; measure,
  do not sprinkle.
- `LazyLock` table accesses in hot paths (1.2.3): replace by initialised
  statics if the profile shows the check.

**End state.** Each accepted change: identical fingerprint and a measured NPS
gain above noise (for example +0.5%).

### 8.3 PEXT slider attacks

**What.** On x86-64 CPUs with fast BMI2 (Intel since Haswell, AMD since Zen 3),
`_pext_u64(occupancy, mask)` computes the table index directly, replacing the
magic multiply.

```rust
#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
pub fn rook_attacks(sq: Square, occupied: u64) -> u64 {
    use std::arch::x86_64::_pext_u64;
    let entry = &ROOK_PEXT[sq.index()];
    ROOK_ATTACKS[entry.offset + _pext_u64(occupied, entry.mask) as usize]
}
```

Since Rust 1.87 many `std::arch` intrinsics are safe to call in code compiled
with the required target feature; if the compiler asks for `unsafe`, add it with
a `// SAFETY:` comment naming the `cfg` that guarantees BMI2. AMD Zen 1 and Zen 2
implement PEXT in microcode and are much slower with it; their users need the
magic build.

**End state.** A PEXT build selected by `target_feature`, perft- and
bench-identical to the magic build, with its NPS difference measured.

### 8.4 CPU tiers, PGO and the release matrix

**What.** Release assets per instruction-set tier, each built with profile-
guided optimisation, from a CI that refuses to publish inconsistent binaries.

**How it is usually done.**

- **Tiers**: for example `x86-64-v2` (POPCNT), `x86-64-v3` (AVX2, BMI2, with
  PEXT), and AArch64, built with `-C target-cpu=…`.
- **PGO** (maintainer-run locally; scripted in CI later):

  ```powershell
  $env:RUSTFLAGS = "-Cprofile-generate=D:/tmp/ws-pgo"
  cargo build --release --target x86_64-pc-windows-msvc
  .\target\x86_64-pc-windows-msvc\release\whitespine.exe bench
  & "$(rustc --print sysroot)\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-profdata.exe" `
      merge -o D:/tmp/ws-pgo/merged.profdata D:/tmp/ws-pgo
  $env:RUSTFLAGS = "-Cprofile-use=D:/tmp/ws-pgo/merged.profdata"
  cargo build --release --target x86_64-pc-windows-msvc
  ```

  Pass `--target` so build scripts are not instrumented. Fathom's C code is not
  optimised with the profile: `RUSTFLAGS` reach only `rustc`, and instrumenting
  the C compiler separately is not worth it for a library that takes a tiny
  share of the time.
- **CI assertions**: every built asset runs `bench` and the job fails unless
  all print the same node count; the release job fails when the tag differs
  from the version in `Cargo.toml`.
- **CPU check at start-up**: an asset built for a tier the CPU lacks should say
  so instead of crashing with an illegal instruction.

**End state.** The release workflow builds, benches and publishes per-tier PGO
assets; the matrix fingerprint assertion is live (proven by a deliberate
mismatch on a test tag).

### 8.5 Final classical refit: evaluation, then search margins

**What.** A last data cycle with the strongest classical head (5.9's
procedure), then a registered SPSA of the search's centipawn margins (5.7's
procedure) on the refitted evaluation, never mixed with evaluation parameters.

**End state.** Refit and margin tune accepted or recorded as not accepting.

### 8.6 Classical checkpoint and release 3.0.0

**What.** Where Whitespine stands against the target defined at the top of
this plan.

**How.** Tier-5 gauntlet at 1T and 4T, at least 400 games per pair, no
adjudication; a 10+0.1 run against 2.4.0; a written summary of where the Elo
came from, phase by phase, from the accepted SPRTs; and a checklist of "The
target" with each item marked met, partly met or not met, with the evidence.

**End state.** Release 3.0.0, the classical Whitespine, and a checkpoint entry
that Phase 9 compares against.

---

## Phase 9 — NNUE

**The idea.** An *efficiently updatable neural network* replaces the
hand-crafted evaluation with a small network whose first layer is so simple
that a move changes it incrementally: each (piece, square) feature adds or
removes one column of weights from an accumulator, and only the tiny layers
after the accumulator are recomputed per node. The network is trained offline
to predict game results and search scores from positions, so it learns what
Phase 4 wrote by hand, and much that nobody has written down.

**The principle.** Networks are trained only on data generated by Whitespine
itself. That keeps the whole chain, from the classical data generator to the
final network, something you built and understand, and it keeps the licence
question simple.

**Why last.** A classical engine is the data generator for the first networks
and the fallback when a network is missing; and every search constant must be
refitted when the evaluation's scale changes, which is only worth doing once.

**Rust you will practise.** `include_bytes!`, fixed-point arithmetic and
quantisation, `core::arch` SIMD intrinsics behind `cfg`, a per-ply stack of
large `Copy` arrays, exhaustive conformance tests against reference values.

**End state of the phase.** A Whitespine release that plays with a network
trained on its own data, beats the classical 3.0.0 at short and long controls
and at four threads, and whose network file format is documented in this
repository.

### 9.1 The network contract

**What.** A written, versioned description of the network that both the
trainer and the engine implement: everything needed to compute the same number
from the same position in two different programs.

**How it is usually done.** The first architecture is the simplest that works
well, a **768-input perspective network**:

- **Inputs**: 2 colours × 6 piece types × 64 squares = 768 features per
  perspective, one perspective from each side's point of view (the board
  flipped for Black), so the network sees "my pieces" and "their pieces"
  regardless of colour.
- **Accumulator**: one hidden layer per perspective, a few hundred neurons
  (256 to 1,024), `i16` weights and biases.
- **Activation**: clipped ReLU (`clamp(0, Q)`) or squared clipped ReLU, which
  is cheaper to train well.
- **Output**: the two activated accumulators concatenated with the side to
  move first, a dot product with the output weights, one output bucket at
  first (later several, chosen by piece count).
- **Quantisation**: the scale factors that turn the trainer's floating-point
  weights into integers (`QA` for the accumulator, `QB` for the output), and
  the scale that turns the integer output into centipawns.
- **File format**: a header with a magic string and the contract version, then
  the weights in a fixed order and byte order.
- **Conformance set**: a few hundred positions with the trainer's reference
  evaluation for each, so the engine's inference can be checked to the integer.

**End state.** `docs/nnue-contract.md` with the architecture, quantisation,
file layout and a version number; a change to any of them bumps the version,
and the engine refuses a file whose version it does not implement.

### 9.2 Board events for the accumulator

**What.** An NNUE's first layer is updated incrementally: a move removes and
adds a few (piece, square) features. `make_move` must report exactly which,
including castling (two pieces), en passant (a victim on another square) and
promotion (a pawn becomes another piece).

**How.** A small "dirty pieces" record filled by the piece primitives
(1.3.1) during `make`, consumed by the evaluator. The classical engine ignores
it.

**End state.** Events recorded and tested (applying them to an empty feature
set reproduces the position's features after every perft move); bench
fingerprint unchanged; NPS cost measured.

### 9.3 Data generation at scale

**What.** Tens of millions of positions in the trainer's format, generated by
the classical head. A network cannot be better than its data: the positions
must cover what games reach, the labels must be honest, and no position may
appear in both training and validation sets.

**How.** 5.1's generator, scaled: more threads, a node or depth budget per
move, deduplication, splits by game, manifests with hashes, and binary output
if the trainer prefers it. Label each position with the search score and the
game result; the trainer blends them.

**End state.** A corpus with a manifest; maintainer-run.

### 9.4 Training pipeline

**What.** Turn data into a network file.

**How.** Use an existing open-source NNUE trainer as an external tool, like
fastchess: running a trainer is not writing engine code, and a GPU trainer is
a project of its own. Configure it to the contract of 9.1 and record every
setting. Optionally write a tiny CPU trainer for a toy network yourself to
understand back-propagation and quantisation; do not expect it to produce the
real networks.

**End state.** A baseline network trained with a recorded configuration, two
seeds per configuration, validation loss reported, and the conformance set of
9.1 produced by the trainer.

### 9.5 Scalar inference and conformance

**What.** Load the network (embedded with `include_bytes!` as the default, and
from an `EvalFile` option) and evaluate a position by a full computation in
plain integer Rust.

```rust
// first layer for one perspective: sum the weight columns of the active features
fn refresh(accumulator: &mut [i16; HIDDEN], net: &Network, features: &[usize]) {
    accumulator.copy_from_slice(&net.feature_bias);
    for &feature in features {
        let column = &net.feature_weights[feature * HIDDEN..(feature + 1) * HIDDEN];
        for (a, w) in accumulator.iter_mut().zip(column) {
            *a += *w;
        }
    }
}
// then activation (clipped or squared-clipped ReLU), concatenate both perspectives
// with the side to move first, dot with the output weights of the chosen bucket, and
// de-quantise with the contract's scale factors.
```

**End state.** Integer-exact agreement with the trainer's reference evaluations
on the conformance positions; the classical evaluation kept as a fallback when
no network is loaded.

### 9.6 Incremental accumulators

**What.** Update the first layer from the board events instead of recomputing
it: one accumulator per ply on a stack, lazily updated from the parent. This
is the "efficiently updatable" part, and it is where the speed comes from: a
refresh touches every active feature (about 30 columns), an update touches two
to four.

**End state.** A randomised test over long playouts in which every move type
occurs: the incremental accumulator equals a full refresh at every ply.

### 9.7 SIMD inference

**What.** The accumulator updates and the activation are the hot loops; vector
instructions process 16 or 32 values at a time.

**How.** `core::arch` intrinsics for AVX2 (and AVX-512 later) on x86-64 and NEON
on AArch64, selected per build tier (8.4) with `#[cfg(target_feature = …)]`;
the scalar path stays as the reference. `std::simd` is still unstable, so stable
Rust uses the intrinsics.

**End state.** SIMD paths bit-identical to scalar on the conformance set;
per-tier NPS measured.

### 9.8 Search re-fit for the network

**What.** The network's scale and error profile differ from the classical
evaluation: margins, correction histories, SEE thresholds in pruning and time
factors need refitting.

**End state.** A registered SPSA of the search's scale-dependent constants;
SPRT against the unfitted network build.

### 9.9 Network ladder and data refresh

**What.** Better networks in steps, one architectural change at a time, each
a new contract version and each gated by games:

1. **Output buckets** by piece count: the output layer specialises per game
   phase.
2. **King buckets with mirroring**: the input features are indexed also by the
   own king's square (grouped into a few regions, mirrored left-right), so the
   network learns king-relative patterns.
3. **Larger hidden layers**, as far as the speed cost is repaid.
4. **Richer inputs** describing piece relations and threats.

And data refreshed by the strongest network engine, in cycles like 5.9.

**End state.** Each network gated against the previous one; the data cycle
stopped at the first refresh that does not accept.

### 9.10 NNUE release 4.0.0

**What.** The first network Whitespine.

**How.** Beat 3.0.0 at 3+0.03, at 10+0.1 and at 4 threads; clean platform
matrix; the contract document matches the shipped file.

**End state.** Release 4.0.0. A CCRL submission is your call.

---

## Appendix — References

- Chess Programming Wiki (chessprogramming.org): bitboards, magic bitboards,
  perft results, SEE, transposition table, null move pruning, LMR, futility
  pruning, singular extensions, history heuristics, the classical evaluation
  terms of Phase 4 (pawn structure, mobility, king safety, passed pawns,
  material imbalance), Texel's tuning method, SPSA, NNUE.
- Syzygy and Fathom: the `tbprobe.h` header comments are the contract.
- J. C. Spall, "An Overview of the Simultaneous Perturbation Method for
  Efficient Optimization" (SPSA).
- Y. Nasu, "Efficiently Updatable Neural-Network-based Evaluation Functions
  for Computer Shogi" (the NNUE idea), and the Chess Programming Wiki's NNUE
  page for the chess adaptation.
- The UCI protocol text shipped in this repository (`uci_specification.txt`).
- fastchess `-help` and weather-factory's `README.md` for the exact options of
  the tools in use.

