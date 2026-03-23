use chess::{Board, CastleRights, Color, Piece};

const PLANES: usize = 17;

pub fn board_to_input(board: &Board) -> Vec<f32> {
    let mut input = vec![0.0; PLANES * 64];

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
            let plane = piece_plane(color, piece);
            input[index(plane, row, col)] = 1.0;
        }
    }

    fill_plane(
        &mut input,
        12,
        if board.side_to_move() == Color::White {
            1.0
        } else {
            0.0
        },
    );
    fill_plane(
        &mut input,
        13,
        if has_kingside_rights(board, Color::White) {
            1.0
        } else {
            0.0
        },
    );
    fill_plane(
        &mut input,
        14,
        if has_queenside_rights(board, Color::White) {
            1.0
        } else {
            0.0
        },
    );
    fill_plane(
        &mut input,
        15,
        if has_kingside_rights(board, Color::Black) {
            1.0
        } else {
            0.0
        },
    );
    fill_plane(
        &mut input,
        16,
        if has_queenside_rights(board, Color::Black) {
            1.0
        } else {
            0.0
        },
    );

    input
}

#[inline]
fn piece_plane(color: Color, piece: Piece) -> usize {
    let offset = if color == Color::White { 0 } else { 6 };
    offset + piece.to_index()
}

#[inline]
fn has_kingside_rights(board: &Board, color: Color) -> bool {
    matches!(
        board.castle_rights(color),
        CastleRights::Both | CastleRights::KingSide
    )
}

#[inline]
fn has_queenside_rights(board: &Board, color: Color) -> bool {
    matches!(
        board.castle_rights(color),
        CastleRights::Both | CastleRights::QueenSide
    )
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
    use std::str::FromStr;

    use chess::Board;

    use super::board_to_input;

    #[test]
    fn board_to_input_sets_state_planes() {
        let board = Board::default();
        let input = board_to_input(&board);

        assert_eq!(input.len(), 17 * 64);
        assert!(input[12 * 64..13 * 64].iter().all(|&value| value == 1.0));
        assert!(input[13 * 64..14 * 64].iter().all(|&value| value == 1.0));
        assert!(input[14 * 64..15 * 64].iter().all(|&value| value == 1.0));
        assert!(input[15 * 64..16 * 64].iter().all(|&value| value == 1.0));
        assert!(input[16 * 64..17 * 64].iter().all(|&value| value == 1.0));
    }

    #[test]
    fn board_to_input_tracks_turn_and_castling_rights() {
        let board = Board::from_str("8/8/8/8/8/8/8/4k2K b - - 0 1").unwrap();
        let input = board_to_input(&board);

        assert!(input[12 * 64..13 * 64].iter().all(|&value| value == 0.0));
        assert!(input[13 * 64..14 * 64].iter().all(|&value| value == 0.0));
        assert!(input[14 * 64..15 * 64].iter().all(|&value| value == 0.0));
        assert!(input[15 * 64..16 * 64].iter().all(|&value| value == 0.0));
        assert!(input[16 * 64..17 * 64].iter().all(|&value| value == 0.0));
    }
}
