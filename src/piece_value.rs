pub struct PieceValue {
    pub pawn_value: f64,   // [cp]
    pub knight_value: f64, // [cp]
    pub bishop_value: f64, // [cp]
    pub rook_value: f64,   // [cp]
    pub queen_value: f64,  // [cp]
}

impl PieceValue {
    pub fn default() -> PieceValue {
        return PieceValue {
            pawn_value: 100.,
            knight_value: 350.,
            bishop_value: 350.,
            rook_value: 525.,
            queen_value: 1000.,
        };
    }
}
