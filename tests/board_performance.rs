use std::hint::black_box;
use std::time::Instant;

use whitespine::board::{Board, STARTING_FEN};

const BENCHMARK_FENS: &[&str] = &[
    STARTING_FEN,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "rnbq1k1r/pppp1ppp/4pn2/8/1b1PP3/2N2N2/PPP2PPP/R1BQKB1R w KQ - 2 5",
    "8/8/3p4/KPp4r/8/8/8/7k w - c6 0 1",
    "rnbqkb1r/pppp1ppp/5n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 3 3",
];

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BenchmarkResult {
    label: &'static str,
    operations: u64,
    seconds: f64,
    operations_per_second: f64,
}

#[test]
fn board_core_operations_self_benchmark() {
    let mut boards = BENCHMARK_FENS
        .iter()
        .map(|fen| Board::from_fen(fen).unwrap())
        .collect::<Vec<_>>();

    let iterations = Iterations::for_profile();
    let results = [
        benchmark(
            "legal moves",
            || custom_legal_moves(&mut boards),
            iterations.legal_moves,
            5,
        ),
        benchmark(
            "captures",
            || custom_capture_gen(&mut boards),
            iterations.captures,
            5,
        ),
        benchmark(
            "make/unmake",
            || custom_make_unmake(&mut boards),
            iterations.make_unmake,
            5,
        ),
        benchmark(
            "check detection",
            || custom_check_detection(&boards),
            iterations.check_detection,
            5,
        ),
        benchmark(
            "perft(4) startpos",
            || {
                let mut board = Board::default();
                board.perft(4)
            },
            iterations.perft,
            1,
        ),
        benchmark(
            "game simulation",
            || custom_game_simulation(&mut boards),
            iterations.game_simulation,
            3,
        ),
    ];

    println!();
    println!("Custom board representation performance");
    println!("{}", "-".repeat(72));
    println!(
        "{:<20} {:>16} {:>11} {:>10}",
        "operation", "ops/s", "operations", "seconds"
    );
    for result in &results {
        assert!(result.operations > 0, "{} produced no work", result.label);
        assert!(
            result.operations_per_second.is_finite() && result.operations_per_second > 0.0,
            "{} produced an invalid throughput",
            result.label
        );
        println!(
            "{:<20} {:>16.0} {:>11} {:>10.4}",
            result.label, result.operations_per_second, result.operations, result.seconds
        );
    }
}

#[derive(Debug, Copy, Clone)]
struct Iterations {
    legal_moves: usize,
    captures: usize,
    make_unmake: usize,
    check_detection: usize,
    perft: usize,
    game_simulation: usize,
}

impl Iterations {
    fn for_profile() -> Iterations {
        if cfg!(debug_assertions) {
            Iterations {
                legal_moves: 20,
                captures: 50,
                make_unmake: 10,
                check_detection: 500,
                perft: 1,
                game_simulation: 3,
            }
        } else {
            Iterations {
                legal_moves: 400,
                captures: 800,
                make_unmake: 200,
                check_detection: 20_000,
                perft: 5,
                game_simulation: 50,
            }
        }
    }
}

fn benchmark(
    label: &'static str,
    mut workload: impl FnMut() -> u64,
    iterations: usize,
    warmups: usize,
) -> BenchmarkResult {
    for _ in 0..warmups {
        black_box(workload());
    }

    let mut best_seconds = f64::INFINITY;
    let mut best_operations = 0;
    for _ in 0..3 {
        let mut operations = 0;
        let started = Instant::now();
        for _ in 0..iterations {
            operations += black_box(workload());
        }
        let seconds = started.elapsed().as_secs_f64().max(f64::EPSILON);
        if seconds < best_seconds {
            best_seconds = seconds;
            best_operations = operations;
        }
    }

    BenchmarkResult {
        label,
        operations: best_operations,
        seconds: best_seconds,
        operations_per_second: best_operations as f64 / best_seconds,
    }
}

fn custom_legal_moves(boards: &mut [Board]) -> u64 {
    boards
        .iter_mut()
        .map(|board| board.generate_legal_moves().len() as u64)
        .sum()
}

fn custom_capture_gen(boards: &mut [Board]) -> u64 {
    boards
        .iter_mut()
        .map(|board| board.generate_legal_captures().len() as u64)
        .sum()
}

fn custom_make_unmake(boards: &mut [Board]) -> u64 {
    let mut ops = 0;
    for board in boards {
        let moves = board.generate_legal_moves();
        for &mv in moves.iter() {
            board.make_move_unchecked(mv);
            black_box(board.occupied());
            board.unmake_move(mv);
            ops += 1;
        }
    }
    ops
}

fn custom_check_detection(boards: &[Board]) -> u64 {
    let mut ops = 0;
    for board in boards {
        black_box(board.is_in_check());
        ops += 1;
    }
    ops
}

fn custom_game_simulation(boards: &mut [Board]) -> u64 {
    let mut ops = 0;
    for board in boards {
        let moves = board.generate_legal_moves();
        for &mv in moves.iter() {
            board.make_move_unchecked(mv);
            let opponent_moves = board.generate_legal_moves();
            ops += opponent_moves.len() as u64;
            board.unmake_move(mv);
        }
    }
    ops
}
