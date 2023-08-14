use chess::{Board, BoardStatus, Game, GameResult, Piece};
use std::path::PathBuf;

use crate::piece_value::PieceValue;

#[derive(Debug, Clone)]
pub struct Heuristic {
    pub fifty_moves_rule: bool,
    pub syzygy_path: Option<PathBuf>,

    draw_value: f64,
    loss_value: f64,
    _win_value: f64,
}

impl Heuristic {
    pub fn default() -> Heuristic {
        Heuristic {
            fifty_moves_rule: true,
            syzygy_path: None,

            draw_value: Heuristic::win_probability_to_pawn_advantage(0.5) * 100., // [cp]
            loss_value: Heuristic::win_probability_to_pawn_advantage(0.) * 100.,  // [cp]
            _win_value: Heuristic::win_probability_to_pawn_advantage(1.) * 100.,  // [cp]
        }
    }

    pub fn evaluate(&self, game: &Game) -> f64 {
        // Evaluate board and return value in centi-pawns.
        if game.current_position().status() != BoardStatus::Ongoing {
            if game.result().unwrap() == GameResult::WhiteCheckmates
                || game.result().unwrap() == GameResult::BlackCheckmates
            {
                return self.loss_value;
            }
            return self.draw_value;
        }

        if game.can_declare_draw() {
            return self.draw_value;
        }

        // TODO: syzygy tablebase evaluation

        return Heuristic::evaluate_internal(&game.current_position());
    }

    fn _pawn_advantage_to_win_probability(pawn_advantage: f64) -> f64 {
        // Calculate winning probability given pawn advantage.
        return 1. / (1. + (10_f64).powf(-pawn_advantage / 4.));
    }

    fn win_probability_to_pawn_advantage(mut win_probability: f64) -> f64 {
        // Calculate pawn advantage given winning probability.
        if win_probability <= 0. {
            win_probability = 1e-9
        } else if win_probability >= 1. {
            win_probability = 1. - 1e-9
        }
        return 4. * (win_probability / (1. - win_probability)).log10();
    }

    fn evaluate_internal(board: &Board) -> f64 {
        let pawns = board.pieces(Piece::Pawn);
        let knights = board.pieces(Piece::Knight);
        let bishops = board.pieces(Piece::Bishop);
        let rooks = board.pieces(Piece::Rook);
        let queens = board.pieces(Piece::Queen);

        let mut player_value: f64 = 0.;
        let mut opponent_value: f64 = 0.;
        let piece_value = PieceValue::default();

        for square in pawns.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.pawn_value;
            } else {
                opponent_value += piece_value.pawn_value;
            }
        }

        for square in knights.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.knight_value;
            } else {
                opponent_value += piece_value.knight_value;
            }
        }

        for square in bishops.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.bishop_value;
            } else {
                opponent_value += piece_value.bishop_value;
            }
        }

        for square in rooks.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.rook_value;
            } else {
                opponent_value += piece_value.rook_value;
            }
        }

        for square in queens.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.queen_value;
            } else {
                opponent_value += piece_value.queen_value;
            }
        }
        
        return player_value - opponent_value;
    }
}
