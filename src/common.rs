use crate::bboard::*;
use crate::state::{ChessState, Piece, Side};

#[inline]
pub fn add_bit(board: &mut BBoard, x: u32, y: u32) {
    if x > 7 {
        return;
    }
    if y > 7 {
        return;
    }

    (*board) |= 1u64 << (x + y * 8) as u64;
}

#[inline]
pub fn has_bit(board: &BBoard, x: u32, y: u32) -> bool {
    if x > 7 {
        return false;
    }
    if y > 7 {
        return false;
    }

    return (*board & (1u64 << (x + y * 8) as u64)) > 0;
}

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

/// return tuple with bitboards:
/// king, rook, empty, no_attack, king_move, rook_move
#[inline]
pub fn castle_tuple_k_r_e_na_km_rm(
    side: Side,
    castle_type: Piece,
) -> (BBoard, BBoard, BBoard, BBoard, BBoard, BBoard) {
    let w_k = (
        0b10000u64,
        0b10000000u64,
        0b1100000u64,
        0b1110000u64,
        0b1010000u64,
        0b10100000u64,
    );
    let w_q = (
        0b10000u64, 0b1u64, 0b1110u64, 0b11100u64, 0b10100u64, 0b1001u64,
    );

    let b_k = (
        0b1000u64.reverse_bits(),
        0b1u64.reverse_bits(),
        0b110u64.reverse_bits(),
        0b1110u64.reverse_bits(),
        0b1010u64.reverse_bits(),
        0b101u64.reverse_bits(),
    );
    let b_q = (
        0b1000u64.reverse_bits(),
        0b10000000u64.reverse_bits(),
        0b1110000u64.reverse_bits(),
        0b111000u64.reverse_bits(),
        0b101000u64.reverse_bits(),
        0b10010000u64.reverse_bits(),
    );

    let result = match side {
        Side::White => match castle_type {
            Piece::King => w_k,
            Piece::Queen => w_q,
            _ => panic!(),
        },

        Side::Black => match castle_type {
            Piece::King => b_k,
            Piece::Queen => b_q,
            _ => panic!(),
        },
    };

    result
}

#[cfg(test)]
mod tests {
    use std::num::Wrapping;

    #[test]
    fn test_pop_count() {
        let test3 = 1u64 << 5 | 1 << 10 | 1 << 20;
        assert_eq!(test3.count_ones(), 3);
    }

    #[test]
    fn test_pop_count_side_bits() {
        let test64 = (Wrapping(1u64) << 63) + Wrapping(1);
        assert_eq!(test64.0.count_ones(), 2);
    }

    #[test]
    fn test_pop_count_all_bits() {
        let test64 = Wrapping(0u64) - Wrapping(1);
        assert_eq!(test64.0.count_ones(), 64);
    }

    #[test]
    fn test_pop_count_0() {
        assert_eq!(0u64.count_ones(), 0);
    }
}
