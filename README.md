# Whitespine

Whitespine is a UCI-compatible chess engine written in Rust.
It started as a Rust re-implementation of [Beast](https://github.com/maelic13/beast), but the projects have since diverged.

Whitespine is only the engine. It does not include a graphical interface, so you should use it with a chess GUI that 
supports the UCI protocol.

## Features

- UCI-compatible engine for standard chess GUIs
- classical handcrafted evaluation
- iterative-deepening negamax search with alpha-beta pruning
- quiescence search, delta pruning, and basic move ordering
- time management for standard UCI time controls
- fixed-depth search and infinite analysis mode
- standalone release binaries for Windows, macOS, and Linux

## Releases

- [Latest release](https://github.com/maelic13/whitespine/releases/latest)
- [All releases](https://github.com/maelic13/whitespine/releases)

Release assets include standalone executables for:
- Windows (x64 and arm64)
- macOS (arm64)
- Linux (x64 and arm64)

## Requirements

To use a release executable:
- a UCI-compatible chess GUI

To build from source:
- Rust and Cargo

## Use With A GUI

1. Download the Whitespine executable for your platform from the latest release.
2. Add it as a new UCI engine in your GUI.
3. Start an analysis session or engine game as usual.

Tested GUIs:
- Arena
- ChessBase/Fritz
- ChessOK Aquarium
- Hiarcs Chess Explorer (Pro)

Other GUIs that support UCI should also work, but they have not been tested.

## Build From Source

Build a release binary with:

```bash
cargo build --release
```

The executable will be created at:
- `target/release/whitespine`
- `target/release/whitespine.exe` on Windows

For quick local testing you can also run:

```bash
cargo run --release
```

## License
GPL-3.0-or-later. See [LICENSE](LICENSE).
