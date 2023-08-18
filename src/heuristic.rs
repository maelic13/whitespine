use chess::{Board, Color, Game, GameResult, Piece, Square};
use std::path::PathBuf;

use crate::piece_value::PieceValue;

#[derive(Debug, Clone)]
pub struct Heuristic {
    pub fifty_moves_rule: bool,
    pub syzygy_path: Option<PathBuf>,

    draw_value: f64,
    loss_value: f64,
    win_value: f64,

    pawn_rank_weight: f64,
    pawn_file_weight: f64,
    pawn_center_weight: f64,
    pawn_distance_weight: f64,

    knight_center_weight: f64,
    knight_distance_weight: f64,

    bishop_center_weight: f64,
    bishop_distance_weight: f64,

    rook_center_weight: f64,
    rook_distance_weight: f64,

    queen_center_weight: f64,
    queen_distance_weight: f64,

    king_center_weight: f64,
    king_distance_weight: f64,
}

impl Heuristic {
    pub fn default() -> Heuristic {
        Heuristic {
            fifty_moves_rule: true,
            syzygy_path: None,

            draw_value: 0.,       // [cp]
            loss_value: -120_00., // [cp]
            win_value: 120_00.,   // [cp]

            pawn_rank_weight: 7.,
            pawn_file_weight: 5.,
            pawn_center_weight: 5.,
            pawn_distance_weight: 5.,

            knight_center_weight: 7.,
            knight_distance_weight: 8.,

            bishop_center_weight: 5.,
            bishop_distance_weight: 8.,

            rook_center_weight: 8.,
            rook_distance_weight: 5.,

            queen_center_weight: 2.,
            queen_distance_weight: 8.,

            king_center_weight: 8.,
            king_distance_weight: 5.,
        }
    }

    pub fn evaluate(&self, game: &Game) -> f64 {
        /* Evaluate board and return value in centi-pawns. */
        if game.can_declare_draw() {
            return 0.;
        }

        let result = game.result();
        let color = game.side_to_move();
        if result.is_some() {
            return match result.unwrap() {
                GameResult::WhiteCheckmates | GameResult::BlackResigns => {
                    if color == Color::White {
                        self.win_value
                    } else {
                        self.loss_value
                    }
                }
                GameResult::WhiteResigns | GameResult::BlackCheckmates => {
                    if color == Color::Black {
                        self.win_value
                    } else {
                        self.loss_value
                    }
                }
                GameResult::Stalemate | GameResult::DrawAccepted | GameResult::DrawDeclared => {
                    self.draw_value
                }
            };
        }

        // TODO: syzygy tablebase evaluation

        return self.evaluate_internal(&game.current_position());
    }

    fn _pawn_advantage_to_win_probability(pawn_advantage: f64) -> f64 {
        /* Calculate winning probability given pawn advantage. */
        return 1. / (1. + (10_f64).powf(-pawn_advantage / 4.));
    }

    fn _win_probability_to_pawn_advantage(mut win_probability: f64) -> f64 {
        /* Calculate pawn advantage given winning probability. */
        if win_probability <= 0. {
            win_probability = 1e-9
        } else if win_probability >= 1. {
            win_probability = 1. - 1e-9
        }
        return 4. * (win_probability / (1. - win_probability)).log10();
    }

    fn evaluate_internal(&self, board: &Board) -> f64 {
        let pawns = board.pieces(Piece::Pawn);
        let knights = board.pieces(Piece::Knight);
        let bishops = board.pieces(Piece::Bishop);
        let rooks = board.pieces(Piece::Rook);
        let queens = board.pieces(Piece::Queen);
        let kings = board.pieces(Piece::King);

        let mut player_value: f64 = 0.;
        let mut opponent_value: f64 = 0.;
        let piece_value = PieceValue::default();

        for square in pawns.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.pawn_value;
                player_value += self.pawn_bonus(
                    square,
                    board.side_to_move(),
                    board.king_square(!board.side_to_move()),
                );
            } else {
                opponent_value += piece_value.pawn_value;
                opponent_value += self.pawn_bonus(
                    square,
                    !board.side_to_move(),
                    board.king_square(board.side_to_move()),
                );
            }
        }

        for square in knights.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.knight_value;
                player_value += self.knight_bonus(square, board.king_square(!board.side_to_move()))
            } else {
                opponent_value += piece_value.knight_value;
                opponent_value += self.knight_bonus(square, board.king_square(board.side_to_move()))
            }
        }

        for square in bishops.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.bishop_value;
                player_value += self.bishop_bonus(square, board.king_square(!board.side_to_move()))
            } else {
                opponent_value += piece_value.bishop_value;
                opponent_value += self.knight_bonus(square, board.king_square(board.side_to_move()))
            }
        }

        for square in rooks.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.rook_value;
                player_value += self.rook_bonus(square, board.king_square(!board.side_to_move()))
            } else {
                opponent_value += piece_value.rook_value;
                opponent_value += self.rook_bonus(square, board.king_square(board.side_to_move()))
            }
        }

        for square in queens.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += piece_value.queen_value;
                player_value += self.queen_bonus(square, board.king_square(!board.side_to_move()))
            } else {
                opponent_value += piece_value.queen_value;
                opponent_value += self.queen_bonus(square, board.king_square(board.side_to_move()))
            }
        }

        for square in kings.into_iter() {
            if board.color_on(square).unwrap() == board.side_to_move() {
                player_value += self.king_bonus(
                    square,
                    board.king_square(!board.side_to_move()),
                    queens.count() == 0,
                )
            } else {
                opponent_value += self.king_bonus(
                    square,
                    board.king_square(board.side_to_move()),
                    queens.count() == 0,
                )
            }
        }

        return player_value - opponent_value;
    }

    fn pawn_bonus(&self, pawn: Square, color: Color, opponent_king: Square) -> f64 {
        /* Evaluation bonus for positions of pawns on board. */

        // rank bonus -> the further forward the pawn, the more of a bonus
        let mut p_bonus =
            (pawn.get_rank().to_index() as f64 - color.to_second_rank().to_index() as f64).abs()
                * self.pawn_rank_weight;

        // file penalty -> central files take none, the closer to rim the less pawn's value
        if pawn.get_file().to_index() < 3 {
            p_bonus -= (3. - pawn.get_file().to_index() as f64) * self.pawn_file_weight;
        } else if pawn.get_file().to_index() > 4 {
            p_bonus -= (pawn.get_file().to_index() as f64 - 4.) * self.pawn_file_weight;
        }

        // occupying center bonus
        p_bonus += Heuristic::occupying_center_bonus(pawn, self.pawn_center_weight);
        // distance from king bonus
        p_bonus +=
            Heuristic::distance_from_king_bonus(pawn, opponent_king, self.pawn_distance_weight);

        return p_bonus;
    }

    fn knight_bonus(&self, knight: Square, opponent_king: Square) -> f64 {
        /* Evaluation bonus for positions knights on board. */

        // occupying center bonus
        let mut k_bonus = Heuristic::occupying_center_bonus(knight, self.knight_center_weight);
        // distance from king bonus
        k_bonus +=
            Heuristic::distance_from_king_bonus(knight, opponent_king, self.knight_distance_weight);

        return k_bonus;
    }

    fn bishop_bonus(&self, bishop: Square, opponent_king: Square) -> f64 {
        /* Evaluation bonus for positions of bishops on board. */

        // occupying center bonus
        let mut b_bonus = Heuristic::occupying_center_bonus(bishop, self.bishop_center_weight);
        // distance from king bonus
        b_bonus +=
            Heuristic::distance_from_king_bonus(bishop, opponent_king, self.bishop_distance_weight);

        return b_bonus;
    }

    fn rook_bonus(&self, rook: Square, opponent_king: Square) -> f64 {
        /* Evaluation bonus for positions of rooks on board. */
        let mut r_bonus = 0.;

        // occupying center files bonus
        if (3usize..5usize).contains(&rook.get_file().to_index()) {
            r_bonus += 3. * self.rook_center_weight;
        } else if (2usize..6usize).contains(&rook.get_file().to_index()) {
            r_bonus += 2. * self.rook_center_weight;
        } else if (1usize..7usize).contains(&rook.get_file().to_index()) {
            r_bonus += self.rook_center_weight;
        }

        // distance from king bonus
        r_bonus +=
            Heuristic::distance_from_king_bonus(rook, opponent_king, self.rook_distance_weight);

        return r_bonus;
    }

    fn queen_bonus(&self, queen: Square, opponent_king: Square) -> f64 {
        /* Evaluation bonus for positions of queens on board. */

        // occupying center bonus
        let mut q_bonus = Heuristic::occupying_center_bonus(queen, self.queen_center_weight);
        // distance from king bonus
        q_bonus +=
            Heuristic::distance_from_king_bonus(queen, opponent_king, self.queen_distance_weight);

        return q_bonus;
    }

    fn king_bonus(&self, king: Square, opponent_king: Square, no_queens: bool) -> f64 {
        /* Evaluation bonus for positions of king on board. */
        let king_center_weight: f64;
        if no_queens {
            king_center_weight = self.king_center_weight;
        } else {
            king_center_weight = -self.knight_center_weight;
        }

        // occupying center bonus
        let mut k_bonus = Heuristic::occupying_center_bonus(king, king_center_weight);

        // distance from king bonus
        k_bonus +=
            Heuristic::distance_from_king_bonus(king, opponent_king, self.king_distance_weight);

        return k_bonus;
    }

    fn occupying_center_bonus(piece: Square, bonus: f64) -> f64 {
        /* Bonus for occupying squares close to center. */
        if (3usize..5usize).contains(&piece.get_rank().to_index())
            && (3usize..5usize).contains(&piece.get_file().to_index())
        {
            return 3. * bonus;
        }
        if (2usize..6usize).contains(&piece.get_rank().to_index())
            && (3usize..5usize).contains(&piece.get_file().to_index())
        {
            return 2. * bonus;
        }
        if (1usize..7usize).contains(&piece.get_rank().to_index())
            && (3usize..5usize).contains(&piece.get_file().to_index())
        {
            return bonus;
        }
        return 0.;
    }

    fn distance_from_king_bonus(piece: Square, king: Square, bonus: f64) -> f64 {
        /* Bonus for distance from opponent's king. */
        let distance = (piece.get_rank().to_index() as f64 - king.get_rank().to_index() as f64)
            .abs()
            + (piece.get_file().to_index() as f64 - king.get_file().to_index() as f64).abs();
        return 14. / distance * bonus - bonus;
    }
}
