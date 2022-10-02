use crate::bboard::*;
use crate::debug::*;
use crate::state::*;

type Scores = [i32; 64];

static B_PAWN: Scores = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

static B_KNIGHT: Scores = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

static B_BISHOP: Scores = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

static B_ROOK: Scores = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

static B_QUEEN: Scores = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

static B_KING: Scores = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

static B_KING_END: Scores = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];

static W_PAWN: Scores = flip_scores(&B_PAWN);
static W_KNIGHT: Scores = flip_scores(&B_KNIGHT);
static W_BISHOP: Scores = flip_scores(&B_BISHOP);
static W_ROOK: Scores = flip_scores(&B_ROOK);
static W_QUEEN: Scores = flip_scores(&B_QUEEN);
static W_KING: Scores = flip_scores(&B_KING);
static W_KING_END: Scores = flip_scores(&B_KING_END);

const fn flip_scores(s: &Scores) -> Scores {
    let result: Scores = [
        s[63], s[62], s[61], s[60], s[59], s[58], s[57], s[56], s[55], s[54], s[53], s[52], s[51],
        s[50], s[49], s[48], s[47], s[46], s[45], s[44], s[43], s[42], s[41], s[40], s[39], s[38],
        s[37], s[36], s[35], s[34], s[33], s[32], s[31], s[30], s[29], s[28], s[27], s[26], s[25],
        s[24], s[23], s[22], s[21], s[20], s[19], s[18], s[17], s[16], s[15], s[14], s[13], s[12],
        s[11], s[10], s[19], s[18], s[7], s[6], s[5], s[4], s[3], s[2], s[1], s[0],
    ];

    result
}

static INDEX_64: Scores = [
    63, 0, 58, 1, 59, 47, 53, 2, 60, 39, 48, 27, 54, 33, 42, 3, 61, 51, 37, 40, 49, 18, 28, 20, 55,
    30, 34, 11, 43, 14, 22, 4, 62, 57, 46, 52, 38, 26, 32, 41, 50, 36, 17, 19, 29, 10, 13, 21, 56,
    45, 25, 31, 35, 16, 9, 12, 44, 24, 15, 8, 23, 7, 6, 5,
];

fn bit_scan_forward(bb: u64) -> i32 {
    use std::num::Wrapping;

    let mul = Wrapping(bb) * Wrapping(0x07EDD5E59A4E28C2u64);
    let idx = (mul.0 >> 58) as usize;
    return INDEX_64[idx];
}

fn position_to_score(scores: &Scores, board: BBoard) -> i32 {
    use std::num::Wrapping;

    let mut result = 0;
    let mut bit;
    let mut board = board;

    while board > 0 {
        bit = board & (Wrapping(0u64) - Wrapping(board)).0;
        let idx = bit_scan_forward(bit) as usize;

        result += scores[idx];

        board ^= bit;
    }

    result
}

impl Demo for Scores {
    fn demo(&self) {
        for y in 0..8 {
            for x in 0..8 {
                let v = self[(7 - y) * 8 + x];
                print!("{:3} ", v);
            }
            println!();
        }
        println!();
    }
}

pub fn evaluate_position(state: &ChessState) -> i32 {
    let mut w = 0i32;
    let mut b = 0i32;

    let bside = state.get_side_state(Side::Black);
    let wside = state.get_side_state(Side::White);

    let mut piece_it = PIECES.iter();

    // rook
    let p = piece_it.next().unwrap();
    b += bside.get_board(*p).count_ones() as i32 * p.value();
    w += wside.get_board(*p).count_ones() as i32 * p.value();

    // queen
    let p = piece_it.next().unwrap();
    b += bside.get_board(*p).count_ones() as i32 * p.value();
    w += wside.get_board(*p).count_ones() as i32 * p.value();

    let is_end_game = b + w < 1800;

    // rest of the figures
    for p in piece_it {
        b += bside.get_board(*p).count_ones() as i32 * p.value();
        w += wside.get_board(*p).count_ones() as i32 * p.value();
    }

    b += position_to_score(&B_PAWN, bside.get_board(Piece::Pawn));
    b += position_to_score(&B_BISHOP, bside.get_board(Piece::Bishop));
    b += position_to_score(&B_KNIGHT, bside.get_board(Piece::Knight));
    b += position_to_score(&B_QUEEN, bside.get_board(Piece::Queen));
    b += position_to_score(&B_ROOK, bside.get_board(Piece::Rook));

    w += position_to_score(&W_PAWN, wside.get_board(Piece::Pawn));
    w += position_to_score(&W_BISHOP, wside.get_board(Piece::Bishop));
    w += position_to_score(&W_KNIGHT, wside.get_board(Piece::Knight));
    w += position_to_score(&W_QUEEN, wside.get_board(Piece::Queen));
    w += position_to_score(&W_ROOK, wside.get_board(Piece::Rook));

    if is_end_game {
        b += position_to_score(&B_KING_END, bside.get_board(Piece::King));
        w += position_to_score(&W_KING_END, wside.get_board(Piece::King));
    } else {
        b += position_to_score(&B_KING, bside.get_board(Piece::King));
        w += position_to_score(&W_KING, wside.get_board(Piece::King));
    }

    w - b
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_scan_forward() {
        for i in 0u64..63 {
            let bb: BBoard = 1u64 << i;
            assert_eq!(bit_scan_forward(bb), i as i32);
        }
    }

    #[test]
    fn test_position_to_score() {
        for i in 0u64..63 {
            let bb: BBoard = 1u64 << i;
            let result = position_to_score(&W_PAWN, bb);

            assert_eq!(result, W_PAWN[i as usize]);
        }

        let bb: BBoard = 1 << 15 | 1 << 7;
        let result = position_to_score(&W_PAWN, bb);

        assert_eq!(result, W_PAWN[15] + W_PAWN[7]);
    }
}
