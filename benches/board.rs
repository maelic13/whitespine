use std::hint::black_box;
use std::time::{Duration, Instant};

use whitespine::board::{Board, generate_captures, generate_legal_moves, perft};

const WARMUP: Duration = Duration::from_millis(150);
const MEASURE: Duration = Duration::from_millis(750);

const BENCHMARK_FENS: &[(&str, &str)] = &[
    (
        "startpos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ),
    (
        "midgame",
        "rnbq1k1r/pppp1ppp/4pn2/8/1b1PP3/2N2N2/PPP2PPP/R1BQKB1R w KQ - 2 5",
    ),
    ("endgame", "8/2p5/3p4/KP5r/8/8/8/7k w - - 0 1"),
    (
        "in-check",
        "rnbqkb1r/pppp1ppp/5n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 3 3",
    ),
];

struct BenchResult {
    label: &'static str,
    unit: &'static str,
    ops: u64,
    iterations: u64,
    elapsed: Duration,
}

impl BenchResult {
    fn ops_per_second(&self) -> f64 {
        self.ops as f64 / self.elapsed.as_secs_f64()
    }
}

fn main() {
    let boards: Vec<Board> = BENCHMARK_FENS
        .iter()
        .map(|(_, fen)| Board::from_fen(fen).unwrap())
        .collect();

    let mut capture_boards = boards.clone();
    let mut mutable_boards = boards.clone();
    let mut see_boards = boards.clone();
    let mut simulation_boards = boards.clone();

    let results = [
        measure("legal movegen", "moves", || legal_movegen(&boards)),
        measure("capture gen", "moves", || capture_gen(&mut capture_boards)),
        measure("make/unmake", "moves", || make_unmake(&mut mutable_boards)),
        measure("check detection", "positions", || check_detection(&boards)),
        measure("see captures", "captures", || see_captures(&mut see_boards)),
        measure("game simulation", "moves", || {
            game_simulation(&mut simulation_boards)
        }),
        measure("perft startpos d4", "nodes", || perft_startpos(4)),
    ];

    println!();
    println!("Whitespine board benchmark");
    println!("positions: {}", BENCHMARK_FENS.len());
    println!("warmup: {} ms", WARMUP.as_millis());
    println!("measure: {} ms per workload", MEASURE.as_millis());
    println!();
    println!(
        "{:<20} {:>16} {:<10} {:>12} {:>12}",
        "workload", "throughput", "unit", "iterations", "time ms"
    );
    println!("{}", "-".repeat(76));

    for result in &results {
        println!(
            "{:<20} {:>16.0} {:<10} {:>12} {:>12}",
            result.label,
            result.ops_per_second(),
            result.unit,
            result.iterations,
            result.elapsed.as_millis()
        );
    }
}

fn measure<F>(label: &'static str, unit: &'static str, mut workload: F) -> BenchResult
where
    F: FnMut() -> u64,
{
    let warmup_start = Instant::now();
    while warmup_start.elapsed() < WARMUP {
        black_box(workload());
    }

    let start = Instant::now();
    let mut ops = 0u64;
    let mut iterations = 0u64;

    while start.elapsed() < MEASURE {
        ops += black_box(workload());
        iterations += 1;
    }

    BenchResult {
        label,
        unit,
        ops,
        iterations,
        elapsed: start.elapsed(),
    }
}

fn legal_movegen(boards: &[Board]) -> u64 {
    boards
        .iter()
        .map(|board| black_box(generate_legal_moves(black_box(board)).len() as u64))
        .sum()
}

fn capture_gen(boards: &mut [Board]) -> u64 {
    boards
        .iter_mut()
        .map(|board| black_box(generate_captures(black_box(board)).len() as u64))
        .sum()
}

fn make_unmake(boards: &mut [Board]) -> u64 {
    let mut ops = 0u64;
    for board in boards {
        let moves = generate_legal_moves(board);
        for mv in moves {
            board.make_move(mv);
            black_box(&board);
            board.unmake_move(mv);
            ops += 1;
        }
    }
    ops
}

fn check_detection(boards: &[Board]) -> u64 {
    boards
        .iter()
        .map(|board| {
            black_box(board.is_in_check());
            1
        })
        .sum()
}

fn see_captures(boards: &mut [Board]) -> u64 {
    let mut ops = 0u64;
    for board in boards {
        let captures = generate_captures(board);
        for &mv in &captures {
            black_box(board.see(mv));
            ops += 1;
        }
    }
    ops
}

fn game_simulation(boards: &mut [Board]) -> u64 {
    let mut ops = 0u64;
    for board in boards {
        let moves = generate_legal_moves(board);
        for mv in moves {
            board.make_move(mv);
            ops += generate_legal_moves(board).len() as u64;
            board.unmake_move(mv);
        }
    }
    ops
}

fn perft_startpos(depth: u32) -> u64 {
    let mut board = Board::starting_position();
    perft(&mut board, depth)
}
