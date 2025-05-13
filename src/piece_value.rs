use chess::Piece;

pub struct PieceValue {
    pub pawn_value: f64,
    pub knight_value: f64,
    pub bishop_value: f64,
    pub rook_value: f64,
    pub queen_value: f64,
    pub king_value: f64,
}

impl PieceValue {
    pub fn default() -> PieceValue {
        PieceValue {
            pawn_value: 100.,
            knight_value: 350.,
            bishop_value: 370.,
            rook_value: 550.,
            queen_value: 950.,
            king_value: f64::INFINITY,
        }
    }

    pub fn get_piece_value(&self, piece: Piece) -> f64 {
        match piece {
            Piece::Pawn => self.pawn_value,
            Piece::Knight => self.knight_value,
            Piece::Bishop => self.bishop_value,
            Piece::Rook => self.rook_value,
            Piece::Queen => self.queen_value,
            Piece::King => self.king_value,
        }
    }
}
