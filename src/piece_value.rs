use chess::Piece;

pub struct PieceValue {
    pub pawn_value: f64,   // [cp]
    pub knight_value: f64, // [cp]
    pub bishop_value: f64, // [cp]
    pub rook_value: f64,   // [cp]
    pub queen_value: f64,  // [cp]
    pub king_value: f64,  // [cp]
}

impl PieceValue {
    pub fn default() -> PieceValue {
        return PieceValue {
            pawn_value: 100.,
            knight_value: 350.,
            bishop_value: 350.,
            rook_value: 525.,
            queen_value: 1000.,
            king_value: f64::INFINITY,
        };
    }
    
    pub fn get_piece_value(&self, piece: Piece) -> f64 {
        return match piece {
            Piece::Pawn => self.pawn_value,
            Piece::Knight => self.knight_value,
            Piece::Bishop => self.bishop_value,
            Piece::Rook => self.rook_value,
            Piece::Queen => self.queen_value,
            Piece::King => self.king_value,
        }
    }
}
