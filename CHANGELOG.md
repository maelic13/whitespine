# Changelog

All notable changes to Whitespine are documented here.

Dates use `YYYY-MM-DD`.

## [2.0.0] - 2026-05-21

### Added

- Added the current custom board implementation.
- Added a dependency-free board implementation benchmark for legal move generation, capture generation, make/unmake, check detection, SEE, game-simulation style movegen, and perft.
- Added the UCI `bench [depth]` command for local performance checks.
- Added broader correctness coverage for board behavior, SEE, evaluation, search, and the transposition table.
- Added quiescence-search transposition table probing and storage.

### Changed

- Bumped the engine version from `1.4.0` to `2.0.0`.
- Removed all third-party Rust crate dependencies and the dependency-backed benchmark/performance comparison code.
- Matched Basilisk's UCI `Hash` maximum of `33554432` MB.
- Refined search with singular extensions, ProbCut, late-move pruning, internal iterative reduction, correction history, and updated move-ordering heuristics.
- Improved evaluation coverage and qsearch correction-history use.
- Updated the README with current feature, UCI option, build, release, and development information.

### Fixed

- Fixed a history-buffer overflow that could occur in long games.
- Fixed SEE behavior and added regression coverage.
- Fixed quiescence search in checked positions so it returns real scores instead of the `VALUE_NONE` sentinel.
- Preserved `VALUE_NONE` correctly when storing static evaluations in the transposition table.

## [1.4.0] - 2026-04-29

### Changed

- Cleaned up engine code.

### Fixed

- Fixed search and heuristic bugs.

## [1.3.3] - 2026-03-26

### Changed

- Improved documentation.
- Improved release infrastructure.

## [1.3.2] - 2025-09-16

### Changed

- Split build platform groups in release automation.
- Removed musl Linux release builds.

### Fixed

- Fixed black-time handling in time management.

## [1.3.1] - 2025-06-08

### Changed

- Improved executable builds.
- Updated piece values.
- Adjusted search to use available increment more effectively.

### Fixed

- Improved draw handling.
- Fixed delta-pruning behavior.

## [1.3.0] - 2025-05-11

### Added

- Added basic move ordering.

## [1.2.0] - 2025-05-11

### Fixed

- Fixed GitHub Actions configuration.
