use std::sync::mpsc::Receiver;

use crate::board::board::Board;
use crate::engine_command::EngineCommand;
use crate::search::{SearchInfo, Searcher};
use crate::search_options::SearchOptions;
use crate::tt::{MATE_SCORE, MAX_PLY, TranspositionTable};

const BENCH_FENS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/p1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
    "8/pp2k3/8/2p5/2P5/1P2K3/P7/8 w - - 0 1",
    "r1bq1r2/pp2n3/4N2k/3pPppP/1b1n2Q1/2N5/PP3PP1/R1B1K2R w KQ g6 0 20",
    "r4rk1/pp1n1pp1/2p1pn1p/q7/3P4/2NB4/PP3PPP/R2QR1K1 w - - 0 1",
    "5k2/5p1p/p3B1p1/Pp6/1P6/5P1P/4K1P1/8 b - - 0 1",
    "6k1/p3q2p/1nr3p1/8/3Q4/7P/PP4P1/4R1K1 b - - 0 1",
    "2r3k1/1q2Rp1p/p2p2p1/1p1P4/1Pp1P3/2Q5/1P4PP/6K1 w - - 0 1",
    "1r3rk1/p4ppp/2p5/3Nb3/1p1bP3/1B4P1/PP3P1P/R2R2K1 b - - 0 1",
    "r2qr1k1/p4ppp/1pn1bn2/2b1p3/4P3/1BN1BN2/PPP2PPP/R2QR1K1 b - - 6 10",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/4P3/2NP1N2/PPP2PPP/R1BQKB1R w KQkq - 0 5",
    "8/8/p1p5/1p5p/1P5P/P1P5/8/K1k5 w - - 0 1",
    "1k6/1b6/8/5p2/p1p2p2/P7/1P3P2/K7 b - - 0 1",
];

pub struct Engine {
    receiver: Receiver<EngineCommand>,
    searcher: Searcher,
    tt: TranspositionTable,
    hash_mb: usize,
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Self {
        Self {
            receiver,
            searcher: Searcher::new(),
            tt: TranspositionTable::new(64),
            hash_mb: 64,
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = match self.receiver.recv() {
                Ok(command) => command,
                Err(_) => break,
            };

            if command.quit {
                break;
            }
            if command.stop {
                continue;
            }
            if command.bench {
                self.run_bench(command.bench_depth.max(1));
                continue;
            }
            if self.go(&command.search_options) {
                break;
            }
        }
    }

    fn go(&mut self, options: &SearchOptions) -> bool {
        if options.hash_mb != self.hash_mb {
            self.tt.resize(options.hash_mb);
            self.hash_mb = options.hash_mb;
        }

        let mut board = options.board.clone();
        let depth_limit = if options.depth == u32::MAX {
            (MAX_PLY - 1) as u32
        } else {
            options.depth.max(1).min((MAX_PLY - 1) as u32)
        };
        let (soft_limit, hard_limit) = self.time_limits(options);

        let receiver = &self.receiver;
        let mut saw_quit = false;
        let mut saw_stop = false;
        let result = self.searcher.iterative_deepening(
            &mut board,
            &mut self.tt,
            depth_limit,
            soft_limit,
            hard_limit,
            || {
                if saw_quit || saw_stop {
                    return true;
                }
                match receiver.try_recv() {
                    Ok(command) if command.quit => {
                        saw_quit = true;
                        true
                    }
                    Ok(command) if command.stop => {
                        saw_stop = true;
                        true
                    }
                    _ => false,
                }
            },
            |info| print_info(info),
        );

        let best_move = if result.best_move.is_null() {
            result
                .pv
                .first()
                .copied()
                .unwrap_or_else(|| first_legal_move(&options.board))
        } else {
            result.best_move
        };
        println!("bestmove {}", best_move);
        saw_quit
    }

    fn run_bench(&mut self, depth: u32) {
        self.searcher.clear_all();
        self.tt.clear();

        let mut total_nodes = 0u64;
        let mut total_time = 0u64;
        println!();
        for (index, fen) in BENCH_FENS.iter().enumerate() {
            let mut board = match Board::from_fen(fen) {
                Ok(board) => board,
                Err(_) => {
                    println!("info string bench: invalid FEN {}", fen);
                    continue;
                }
            };
            let result = self.searcher.iterative_deepening(
                &mut board,
                &mut self.tt,
                depth,
                u64::MAX,
                u64::MAX,
                || false,
                |_| {},
            );

            let time_ms = result.time_ms.max(1);
            let nps = result.nodes * 1000 / time_ms;
            total_nodes += result.nodes;
            total_time += result.time_ms;
            println!(
                "bench {}/{}  depth {}  score {}  nodes {}  time {}ms  nps {}",
                index + 1,
                BENCH_FENS.len(),
                result.depth,
                format_score(result.score),
                result.nodes,
                result.time_ms,
                nps,
            );
        }

        let total_nps = if total_time > 0 {
            total_nodes * 1000 / total_time
        } else {
            total_nodes
        };
        println!(
            "
========================="
        );
        println!("Total time (ms) : {}", total_time);
        println!("Nodes searched  : {}", total_nodes);
        println!("Nodes/second    : {}", total_nps);
    }

    fn time_limits(&self, options: &SearchOptions) -> (u64, u64) {
        if options.move_time > 0 {
            let move_time = options
                .move_time
                .saturating_sub(options.move_overhead)
                .max(1);
            return (move_time, move_time);
        }

        let (time, inc) = if options.board.side_to_move as usize == 0 {
            (options.white_time, options.white_increment)
        } else {
            (options.black_time, options.black_increment)
        };
        if time == 0 {
            return (u64::MAX, u64::MAX);
        }

        let available = time.saturating_sub(options.move_overhead).max(1);
        let base = available / 25;
        let soft = (base + (inc * 3) / 4).max(1);
        let hard = ((available / 2).min(soft.saturating_mul(3))).max(soft);
        (soft, hard)
    }
}

fn print_info(info: SearchInfo) {
    let pv = info
        .pv
        .iter()
        .map(|mv| mv.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "info depth {} seldepth {} score {} nodes {} nps {} hashfull {} time {} pv {}",
        info.depth,
        info.seldepth,
        format_score(info.score),
        info.nodes,
        info.nps,
        info.hashfull,
        info.time_ms,
        pv,
    );
}

fn format_score(score: i32) -> String {
    if score.abs() >= MATE_SCORE - MAX_PLY as i32 {
        let mate = ((MATE_SCORE - score.abs()) + 1) / 2;
        if score > 0 {
            format!("mate {}", mate)
        } else {
            format!("mate -{}", mate)
        }
    } else {
        format!("cp {}", score)
    }
}

fn first_legal_move(board: &Board) -> crate::board::moves::Move {
    crate::board::movegen::generate_legal_moves(board)
        .into_iter()
        .next()
        .unwrap_or(crate::board::moves::Move::NULL)
}
