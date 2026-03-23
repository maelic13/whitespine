use chess::{Board, Color, Piece};

const PLANES: usize = 7;

pub fn board_to_input(board: &Board) -> Vec<f32> {
    let mut input = vec![0.0; PLANES * 64];
    let turn = if board.side_to_move() == Color::White {
        1.0
    } else {
        -1.0
    };
    fill_plane(&mut input, 6, turn);

    for piece in [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ] {
        for square in *board.pieces(piece) {
            let Some(color) = board.color_on(square) else {
                continue;
            };

            let row = 7 - square.get_rank().to_index();
            let col = square.get_file().to_index();
            let plane = piece.to_index();
            let sign = if color == Color::White { 1.0 } else { -1.0 };
            input[index(plane, row, col)] = sign;
        }
    }

    input
}

#[inline]
fn fill_plane(input: &mut [f32], plane: usize, value: f32) {
    let start = plane * 64;
    let end = start + 64;
    input[start..end].fill(value);
}

#[inline]
fn index(plane: usize, row: usize, col: usize) -> usize {
    plane * 64 + row * 8 + col
}

#[cfg(test)]
mod tests {
    use chess::{Board, Color, Square};

    use super::board_to_input;

    #[test]
    fn board_to_input_sets_turn_and_piece_planes() {
        let board = Board::default();
        let input = board_to_input(&board);

        assert_eq!(input.len(), 7 * 64);

        for idx in 6 * 64..7 * 64 {
            assert_eq!(input[idx], 1.0);
        }

        let a2 = Square::A2;
        let a7 = Square::A7;

        let a2_idx = (7 - a2.get_rank().to_index()) * 8 + a2.get_file().to_index();
        let a7_idx = (7 - a7.get_rank().to_index()) * 8 + a7.get_file().to_index();

        assert_eq!(board.color_on(a2), Some(Color::White));
        assert_eq!(board.color_on(a7), Some(Color::Black));
        assert_eq!(input[a2_idx], 1.0);
        assert_eq!(input[a7_idx], -1.0);
    }
}
