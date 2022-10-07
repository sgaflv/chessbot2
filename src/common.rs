use crate::bboard::*;
use crate::state::{ChessState, BBPiece, Side};

/* 

pub fn update_castles(state: &mut ChessState, side: Side) {
    let side_state = state.get_mut_side_state(side);

    if side_state.king_side_castle {
        let (king_b, rook_b, _, _, _, _) = castle_tuple_k_r_e_na_km_rm(side, Piece::King);

        if side_state.get_board(Piece::King) & king_b == 0 {
            side_state.king_side_castle = false;
            side_state.queen_side_castle = false;
            return;
        }

        if side_state.get_board(Piece::Rook) & rook_b == 0 {
            side_state.king_side_castle = false;
        }
    }

    if side_state.queen_side_castle {
        let (king_b, rook_b, _, _, _, _) = castle_tuple_k_r_e_na_km_rm(side, Piece::Queen);

        if side_state.get_board(Piece::King) & king_b == 0 {
            side_state.king_side_castle = false;
            side_state.queen_side_castle = false;
            return;
        }

        if side_state.get_board(Piece::Rook) & rook_b == 0 {
            side_state.queen_side_castle = false;
        }
    };
}
*/
