# Whitespine

Whitespine is a UCI-compatible chess engine written in Rust.
It began as a Rust re-implementation of [Beast](https://github.com/maelic13/beast), but the engine has since diverged.

Current version: 2.0.0.

Whitespine is engine-only software. It does not include a graphical interface, so use it through a chess GUI or another UCI-compatible frontend.

## Features

- UCI protocol support for standard chess GUIs
- Classical handcrafted evaluation
- Iterative-deepening negamax search with alpha-beta pruning
- Transposition table, aspiration windows, null-move pruning, and selective extensions
- Quiescence search with capture ordering and delta pruning
- Main, capture, continuation, killer, and countermove history heuristics
- Fixed-depth, movetime, clock-managed, and infinite analysis modes
- Configurable hash table and move overhead
- No third-party Rust crate dependencies
- Standalone release binaries for Windows, macOS, and Linux

## Releases

Download prebuilt binaries from:

- [Latest release](https://github.com/maelic13/whitespine/releases/latest)
- [All releases](https://github.com/maelic13/whitespine/releases)

Release assets are built for:

- Windows x64 and arm64
- macOS arm64
- Linux x64 and arm64

## Requirements

To run a release binary:

- A UCI-compatible chess GUI

To build from source:

- Rust and Cargo

## Use With A GUI

1. Download the Whitespine executable for your platform.
2. Add it as a UCI engine in your GUI.
3. Start an engine game, analysis session, or tournament as usual.

Tested GUIs:

- Arena
- ChessBase/Fritz
- ChessOK Aquarium
- Hiarcs Chess Explorer Pro

Other UCI-compatible GUIs should work, but they have not been tested.

## UCI Options

Whitespine exposes these UCI options:

- `Hash`: transposition table size in MB, default `64`, range `1` to `33554432`
- `Move Overhead`: reserved time per move in milliseconds, default `10`, range `0` to `5000`
- `Threads`: currently fixed at `1`

Supported search commands include fixed depth, movetime, standard clock controls, increments, and infinite analysis. Examples:

```text
go depth 8
go movetime 1000
go wtime 300000 btime 300000 winc 2000 binc 2000
go infinite
```

The engine also accepts `bench [depth]` for local performance checks.

## Build From Source

Build an optimized binary with:

```bash
cargo build --release
```

The executable is written to:

- `target/release/whitespine`
- `target/release/whitespine.exe` on Windows

Run the engine locally with:

```bash
cargo run --release
```

Run the board implementation benchmark with:

```bash
cargo bench --bench board
```

This benchmark uses only Whitespine's board code. It does not depend on Criterion or compare against external chess libraries.

## Development

Run focused correctness checks with:

```bash
cargo test --test board_correctness
cargo test --test search
cargo test --test eval
cargo test --test transposition_table
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
