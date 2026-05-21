pub const DEFAULT_BENCH_DEPTH: u16 = 13;

// 16 positions covering openings, middlegames, endgames, and tactics.
// The final "Nodes searched" total acts as a search fingerprint.
pub const BENCH_FENS: [&str; 16] = [
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
