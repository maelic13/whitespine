pub mod attacks;
pub mod bitboard;
pub mod board;
pub mod movegen;
pub mod moves;
pub mod piece;
pub mod square;
pub mod zobrist;

// Convenient re-exports of the most commonly used types.
pub use attacks::ATTACKS;
pub use bitboard::Bitboard;
pub use board::{Board, GameResult, STARTING_FEN};
pub use movegen::{generate_captures, generate_legal_movelist, generate_legal_moves, perft};
pub use moves::{Move, MoveList};
pub use piece::{CastlingRights, Color, Piece};
pub use square::{File, Rank, Square};
pub use zobrist::ZOBRIST;
