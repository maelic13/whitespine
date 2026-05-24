# Changelog

All notable changes to Whitespine are documented in this file.

## [1.4.1] - 2026-05-23

### Fixed

- Held `go ponder` results until `ponderhit` or `stop` so GUIs do not receive
  a premature `bestmove` while the engine is still expected to ponder.
- Held `go infinite` results until `stop` or `quit`.
- Ignored normal move-clock expiry while still pondering and restarted the move
  timer on `ponderhit`.
- Added a final legality check before emitting `bestmove`; if the principal
  variation somehow contains a move that is not legal in the searched root
  position, the engine falls back to another legal move or `0000` when no legal
  move exists.

### Added

- Regression coverage for ponder/infinite UCI flags, ponder timer handling,
  final bestmove legality fallback, and the final positions from the supplied
  illegal-move PGNs.

### Notes

- The supplied `GrandTour.pgn` contains two `rules infraction` games. The legal
  PGN stops with the Stockfish side to move in both cases. The standalone
  `game.pgn` is the same first final position but has a contradictory result
  header. No illegal move text is present in either PGN, only Arena's
  `Illegal move!` annotation.
