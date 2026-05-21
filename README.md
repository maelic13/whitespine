# Whitespine

Whitespine is a UCI-compatible chess engine written in Rust.
The engine is intended for use from a chess GUI or engine-testing tool that
speaks the UCI protocol.

## Highlights

- Custom bitboard board representation with incremental make/unmake
- Legal move generation for all standard chess rules, including castling,
  en passant, promotions, repetition, the fifty-move rule, and insufficient
  material detection
- Zobrist hashing, transposition table, and pawn evaluation cache
- Iterative deepening negamax/PVS search with aspiration windows
- Configurable Lazy SMP-style parallel search with persistent workers and a
  full-key validated shared transposition table through the UCI `Threads`
  option
- Quiescence search with delta pruning, checks/promotions in early qsearch, and
  SEE-based pruning
- Null-move pruning, ProbCut, singular extensions, futility pruning, late move
  pruning, and late move reductions
- Move ordering using TT moves, SEE, killers, countermoves, main history,
  capture history, and continuation history
- Correction history and handcrafted tapered evaluation
- Built-in `bench` UCI command for repeatable search benchmarks

## UCI Support

Supported commands include:

- `uci`
- `isready`
- `ucinewgame`
- `position startpos [moves ...]`
- `position fen <fen> [moves ...]`
- `go` with `depth`, `nodes`, `movetime`, `wtime`, `btime`, `winc`, `binc`,
  `movestogo`, `ponder`, and `infinite`
- `stop`
- `ponderhit`
- `quit`
- `bench [depth]`

Supported options:

- `Hash` default `64`
- `Clear Hash`
- `Move Overhead` default `10`
- `Threads` default `1`, min `1`, max `1024`

## Bench

Run the built-in benchmark from a UCI session:

```text
bench
bench 13
```

The bench command searches a fixed suite of positions and reports a repeatable
search fingerprint and speed data. It is useful for comparing local changes,
compiler settings, and machine performance.

The benchmark uses the current UCI options, including `Threads`, so a threaded
search benchmark can be run with:

```text
setoption name Threads value 8
bench
```

Run the board implementation benchmark with:

```bash
cargo bench --bench board
```

This benchmark matches the board benchmark used on the `claude` branch. It
measures legal move generation, capture generation, make/unmake, check
detection, SEE over captures, game-simulation-style move generation, and
start-position perft depth 4.

## Build From Source

Install Rust and Cargo, then build an optimized release binary:

```bash
cargo build --release
```

The executable is created at:

- `target/release/whitespine`
- `target/release/whitespine.exe` on Windows

Release builds use LTO and a single codegen unit for engine speed.

For quick local testing:

```bash
cargo run --release
```

## Test

Run the release test suite:

```bash
cargo test --release
```

The suite covers:

- FEN parsing and round-tripping
- Legal move generation and special moves
- Perft reference positions
- Hashing and make/unmake correctness
- Draw and terminal-result handling
- Search limits and stop/quit behavior
- Single-thread determinism and thread-count reconfiguration
- Threaded search node-limit handling
- UCI command ordering, priority quit/stop handling, and stale-search
  cancellation
- Quiet/capture move-generation partitioning
- Evaluation and transposition table behavior
- UCI command handling

## Use With A GUI

1. Build or download a Whitespine executable.
2. Add it as a UCI engine in your chess GUI.
3. Configure `Hash` and `Move Overhead` as needed.
4. Start an engine game or analysis session.

Tested GUI families include Arena, ChessBase/Fritz, ChessOK Aquarium, and
Hiarcs Chess Explorer. Other UCI-compatible GUIs should also work.

## Releases

- [Latest release](https://github.com/maelic13/whitespine/releases/latest)
- [All releases](https://github.com/maelic13/whitespine/releases)

Release assets may include standalone executables for Windows, macOS, and Linux.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
