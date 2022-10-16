use std::fmt;
use std::fmt::Formatter;

use crate::bboard::*;
use crate::debug::Demo;
use crate::game_setup::ChessMove;

pub enum CastleSide {
    Queen,
    King,
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Debug)]
pub enum BBPiece {
    WKing = 0,
    WPawn,
    WRook,
    WKnight,
    WBishop,
    WQueen,
    WAll,
    WAttacks,
    WCastles,
    WEnPassant,
    BKing,
    BPawn,
    BRook,
    BKnight,
    BBishop,
    BQueen,
    BAll,
    BAttacks,
    BCastles,
    BEnPassant,
}

const BBPIECE_MIDDLE: usize = BBPiece::BKing as usize;
const BBPIECE_COUNT: usize = BBPIECE_MIDDLE * 2;

impl BBPiece {

    #[inline]
    pub fn idx(&self) -> usize {
        (*self) as usize
    }

    #[inline]
    pub fn opposite_idx(&self) -> usize {
        let ret = (*self) as usize;

        if ret < BBPIECE_MIDDLE { 
            ret + BBPIECE_MIDDLE
        } else {
            ret - BBPIECE_MIDDLE
        }
    }

    #[inline]
    pub fn value(&self) -> i32 {

        let values = [
            40000, // King
            100,   // Pawn
            500,   // Rook
            320,   // Knight
            330,   // Bishop
            900,   // Queen
            0,     // all
            0,     // WAttack
            0,     // WCastles
            0,     // WEnPassant
            -40000, // king
            -100,   // pawn
            -500,   // rook
            -320,   // knight
            -330,   // bishop
            -900,   // queen
            -0,     // all
            -0,     // BAttack
            -0,     // BCastles
            -0,     // BEnPassant
        ];

        let idx = *self as usize;

        values[idx]

    }

    #[inline]
    pub fn get_pieces() -> [BBPiece; 12] {
        [
            BBPiece::WKing,
            BBPiece::WPawn,
            BBPiece::WRook,
            BBPiece::WKnight,
            BBPiece::WBishop,
            BBPiece::WQueen,
            BBPiece::BKing,
            BBPiece::BPawn,
            BBPiece::BRook,
            BBPiece::BKnight,
            BBPiece::BBishop,
            BBPiece::BQueen,
        ]
    }

    #[inline]
    pub fn to_string(&self) -> String {
        match self {
            BBPiece::WKing => "King".to_string(),
            BBPiece::WPawn => "Pawn".to_string(),
            BBPiece::WRook => "Rook".to_string(),
            BBPiece::WKnight => "Knight".to_string(),
            BBPiece::WBishop  => "Bishop".to_string(),
            BBPiece::WQueen  => "Queen".to_string(),
            BBPiece::BKing => "king".to_string(),
            BBPiece::BPawn => "pawn".to_string(),
            BBPiece::BRook => "rook".to_string(),
            BBPiece::BKnight => "knigt".to_string(),
            BBPiece::BBishop  => "bishop".to_string(),
            BBPiece::BQueen  => "queen".to_string(),
            _ => "".to_string(),
        }
    }

    #[inline]
    pub fn to_char(&self) -> char {
        match *self {
            BBPiece::WKing => 'K',
            BBPiece::WPawn => 'P',
            BBPiece::WRook => 'R',
            BBPiece::WKnight => 'N',
            BBPiece::WBishop  => 'B',
            BBPiece::WQueen  => 'Q',
            BBPiece::BKing => 'k',
            BBPiece::BPawn => 'p',
            BBPiece::BRook => 'r',
            BBPiece::BKnight => 'n',
            BBPiece::BBishop  => 'b',
            BBPiece::BQueen  => 'q',
            _ => '*',
        }
    }

    #[inline]
    pub fn from_byte(c: &u8) -> BBPiece {
        match *c {
            b'P' => BBPiece::WPawn,
            b'R' => BBPiece::WRook,
            b'N' => BBPiece::WKnight,
            b'B' => BBPiece::WBishop,
            b'Q' => BBPiece::WQueen,
            b'K' => BBPiece::WKing,
            b'p' => BBPiece::BPawn,
            b'r' => BBPiece::BRook,
            b'n' => BBPiece::BKnight,
            b'b' => BBPiece::BBishop,
            b'q' => BBPiece::BQueen,
            b'k' => BBPiece::BKing,
            _ => panic!(),
        }
    }

    #[inline]
    pub fn get_side(&self) -> Side {
        if *self < BBPiece::BKing {
            Side::White
        } else {
            Side::Black
        }
    }
    
}

static INITIAL_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Debug, Hash)]
pub enum Side {
    White = 1,
    Black = -1,
}

impl Side {
    #[inline]
    pub fn offset(&self) -> usize {
        match self {
            Side::White => 0,
            Side::Black => BBPIECE_MIDDLE,
        }
    }

    #[inline]
    pub fn offsets(&self) -> (usize, usize) {
        match self {
            Side::White => (0, BBPIECE_MIDDLE),
            Side::Black => (BBPIECE_MIDDLE, 0),
        }
    }

    #[inline]
    pub fn idx(&self) -> usize {
        match self {
            Side::White => 0,
            Side::Black => 1,
        }
    }

    #[inline]
    pub fn value(&self) -> i32 {
        match self {
            Side::White => 1,
            Side::Black => -1,
        }
    }

    #[inline]
    pub fn opposite(&self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }

    pub fn from_byte(c: u8) -> Side {
        match c {
            b'w' => Side::White,
            b'b' => Side::Black,
            _ => panic!("Unknown side character {}", c as char),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Side::White => 'w',
            Side::Black => 'b',
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}


#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ChessState {
    pub next_to_move: Side,

    bboards: [BBoard; BBPIECE_COUNT],

    pub half_move_count: u32,
    pub full_move_count: u32,
}

impl ChessState {
    pub fn new_empty() -> ChessState {
        ChessState {
            next_to_move: Side::White,
            bboards: [0u64; BBPIECE_COUNT],
            half_move_count: 0,
            full_move_count: 0,
        }
    }

    #[inline]
    pub fn piece_at(&self, move_to: BBoard) -> BBPiece {

        if move_to & self.bboard(BBPiece::WAll) > 0 {
            if self.bboards[BBPiece::WPawn as usize] & move_to > 0 {
                return BBPiece::WPawn;
            }
            if self.bboards[BBPiece::WRook as usize] & move_to > 0 {
                return BBPiece::WRook;
            }
            if self.bboards[BBPiece::WKnight as usize] & move_to > 0 {
                return BBPiece::WKnight;
            }
            if self.bboards[BBPiece::WBishop as usize] & move_to > 0 {
                return BBPiece::WBishop;
            }
            if self.bboards[BBPiece::WQueen as usize] & move_to > 0 {
                return BBPiece::WQueen;
            }
            if self.bboards[BBPiece::WKing as usize] & move_to > 0 {
                return BBPiece::WKing;
            }
        }

        if move_to & self.bboard(BBPiece::BAll) > 0 {
            if self.bboards[BBPiece::BPawn as usize] & move_to > 0 {
                return BBPiece::BPawn;
            }
            if self.bboards[BBPiece::BRook as usize] & move_to > 0 {
                return BBPiece::BRook;
            }
            if self.bboards[BBPiece::BKnight as usize] & move_to > 0 {
                return BBPiece::BKnight;
            }
            if self.bboards[BBPiece::BBishop as usize] & move_to > 0 {
                return BBPiece::BBishop;
            }
            if self.bboards[BBPiece::BQueen as usize] & move_to > 0 {
                return BBPiece::BQueen;
            }
            if self.bboards[BBPiece::BKing as usize] & move_to > 0 {
                return BBPiece::BKing;
            }
        }

        panic!("Piece not found at the given position!");
    }

    #[inline]
    pub fn bboard(&self, board: BBPiece) -> BBoard {
        self.bboards[board.idx()]
    }

    #[inline]
    pub fn bboard_mut(&mut self, board: BBPiece) -> &mut BBoard {
        &mut self.bboards[board.idx()]
    }

    #[inline]
    pub fn bboard_ofs(&mut self, board: BBPiece, offset: usize) -> BBoard {
        debug_assert!(board.get_side() == Side::White);
        self.bboards[board.idx() + offset]
    }

    pub fn new_game() -> ChessState {
        ChessState::from_fen(INITIAL_BOARD)
    }

    pub fn do_move(&mut self, chess_move: &ChessMove) {
        for delta in chess_move.deltas.iter() {
            *self.bboard_mut(delta.0) ^= delta.1;
        }

        self.next_to_move = self.next_to_move.opposite();

        if self.next_to_move == Side::White {
            self.full_move_count += 1;
        }
        self.half_move_count += 1;

    }

    pub fn undo_move(&mut self, chess_move: &ChessMove) {
        for delta in chess_move.deltas.iter() {
            *self.bboard_mut(delta.0) ^= delta.1;
        }

        if self.next_to_move == Side::White {
            self.full_move_count -= 1;
        }
        self.half_move_count -= 1;

        self.next_to_move = self.next_to_move.opposite();
    }

    pub fn castle_state(&self, side: Side) -> (bool, bool) {
        let bcastles = if side == Side::White {
            self.bboard(BBPiece::WCastles)
        } else {
            self.bboard(BBPiece::BCastles)
        };

        (bcastles & 1 > 0, bcastles & 2 > 0)
    }

    pub fn castle_moves(&self, side: Side) -> (BBoard, BBoard) {
        let (kcastle, qcastle) = self.castle_state(side);

        if side == Side::White {
            let king_side_castle = if kcastle {
                0b01000000u64
            } else {
                0u64
            };
    
            let queen_side_castle = if qcastle {
                0b00000100u64
            } else {
                0u64
            };
            
            return (king_side_castle, queen_side_castle);
        } else {
            let king_side_castle = if kcastle {
                0b00000010u64.reverse_bits()
            } else {
                0u64
            };
    
            let queen_side_castle = if qcastle {
                0b00100000u64.reverse_bits()
            } else {
                0u64
            };

            return (king_side_castle, queen_side_castle);   
        }
        
    }

    pub fn set_en_passant(&mut self, en_passant: BBoard) {
        if en_passant < 1u64 << 16 {
            *self.bboard_mut(BBPiece::WEnPassant) = en_passant;
        } else {
            *self.bboard_mut(BBPiece::BEnPassant) = en_passant;
        }
    }

    pub fn set_castle_state(&mut self, side: Side, state: (bool, bool)) {
        let mut new_state: BBoard = 0;
        if state.0 {new_state += 1};
        if state.1 {new_state += 2};

        if side == Side::White {
            *self.bboard_mut(BBPiece::WCastles) = new_state;
        } else {
            *self.bboard_mut(BBPiece::BCastles) = new_state;
        };

    }

    pub fn set_king_side_castle(&mut self, side: Side, value: bool) {
        let mut new_state = 0;
        if value {new_state += 1};

        if side == Side::White {
            *self.bboard_mut(BBPiece::WCastles) = new_state;
        } else {
            *self.bboard_mut(BBPiece::BCastles) = new_state;
        };
    }

    pub fn set_queen_side_castle(&mut self, side: Side, value: bool) {
        let mut new_state = 0;
        if value {new_state += 2};

        if side == Side::White {
            *self.bboard_mut(BBPiece::WCastles) = new_state;
        } else {
            *self.bboard_mut(BBPiece::BCastles) = new_state;
        };
    }

    pub fn to_fen(&self) -> String {
        let mut board = ['.'; 64];

        for piece in BBPiece::get_pieces() {
            let bit_board = self.bboards[piece.idx()];
            let c = piece.to_char();

            for i in 0..64 {
                if (bit_board >> i) & 1 == 1 {
                    board[i] = c;
                }
            }
        }

        let mut result = String::new();
        let mut count = 0;

        for y in 0..8 {
            for x in 0..8 {
                let idx = (7 - y) * 8 + x;

                if board[idx] == '.' {
                    count += 1;
                    continue;
                } else if count > 0 {
                    result.push((count + b'0') as char);
                    count = 0;
                }

                result.push(board[idx]);
            }

            if count > 0 {
                result.push((count + b'0') as char);
            }
            count = 0;

            if y < 7 {
                result.push('/')
            }
        }

        result.push_str(
            format!(
                " {} {} {} {} {}",
                self.next_to_move,
                self.castle_string(),
                self.en_passant_string(),
                self.half_move_count,
                self.full_move_count
            )
            .as_str(),
        );

        result
    }

    pub fn castle_string(&self) -> String {
        // get castle string
        let mut castle = String::from("");

        let wc = self.castle_state(Side::White);
        let bc = self.castle_state(Side::Black);

        if wc.0 {
            castle.push('K');
        }
        if wc.1 {
            castle.push('Q');
        }
        if bc.0 {
            castle.push('k');
        }
        if bc.1 {
            castle.push('q');
        }

        castle
    }

    pub fn en_passant_string(&self) -> String {
        bb_to_coord(self.bboard(BBPiece::BEnPassant) | self.bboard(BBPiece::WEnPassant))
    }

    pub fn from_fen(fen: &str) -> ChessState {
        let mut state = ChessState::new_empty();

        let split: Vec<&str> = fen.trim().split(" ").collect();

        if let Some(board) = split.get(0) {
            // parse board pieces
            let mut idx = 0u32;
            for c in board.as_bytes() {
                if idx > 63 {
                    panic!();
                }

                let x = idx % 8;
                let y = 7 - idx / 8;

                if *c == b'/' {
                    // just ignore this symbol for now
                    continue;
                } else if *c >= b'1' && *c <= b'8' {
                    idx += (*c - b'0') as u32;
                    continue;
                } else if b"PpRrNnBbQqKk".contains(c) {
                    let p = BBPiece::from_byte(c);

                    let board: & mut BBoard = state.bboard_mut(p);

                    add_bit(board, x, y);
                } else {
                    panic!("unrecognized FEN board input: {}", board);
                }

                idx = idx + 1;
            }
        }

        if let Some(side) = split.get(1) {
            state.next_to_move = Side::from_byte(side.as_bytes()[0]);
        }

        if let Some(castles) = split.get(2) {
            let (mut bk, mut wk, mut bq, mut wq) = (false, false, false, false);

            for c in castles.as_bytes() {
                match *c {
                    b'K' => wk = true,
                    b'k' => bk = true,
                    b'Q' => wq = true,
                    b'q' => bq = true,

                    _ => {}
                }
            }

            state.set_castle_state(Side::White, (wk, wq));
            state.set_castle_state(Side::Black, (bk, bq));

        }

        if let Some(en_passant) = split.get(3) {
            if en_passant.len() == 2 {
                let bytes = en_passant.as_bytes();
                let x = bytes[0];
                let y = bytes[1];

                let idx = (y - b'1') as u64 * 8 + (x - b'a') as u64;
                let bb_en_passant = 1u64 << idx;

                state.set_en_passant(bb_en_passant);
            }
        }

        if let Some(moves) = split.get(4) {
            state.half_move_count = moves.parse().unwrap();
        }

        if let Some(moves) = split.get(5) {
            state.full_move_count = moves.parse().unwrap();
        }

        state
    }

}

impl Demo for ChessState {

    fn demo(&self) {
        println!("{}", self.to_fen());

        let mut result = ['.'; 64];

        for piece in BBPiece::get_pieces() {

            let board = self.bboard(piece);
            let c = piece.to_char();

            for i in 0..64 {
                if (board >> i) & 1 == 1 {
                    if result[i] != '.' {
                        println!(
                            "board conflict between {} and {} at index {}",
                            result[i], c, i
                        );
                    }
                    result[i] = c;
                }
            }
        }

        // print board
        for y in 0..8 {
            for x in 0..8 {
                let idx = (7 - y) * 8 + x;
                print!("{}", result[idx]);
            }

            println!();
        }

        println!(
            "{} {} {}\n",
            self.next_to_move,
            self.castle_string(),
            self.en_passant_string()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_states() {
        let state1 = ChessState::new_game();
        let state2 =
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let state3 =
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Qkq - 0 1");
        let state4 =
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
        let state5 =
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 0 1");
        let state6 =
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq a3 0 1");
        let state7 =
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 2");

        assert_eq!(state1, state2);
        assert_ne!(state2, state3);
        assert_ne!(state1, state3);
        assert_ne!(state1, state4);
        assert_ne!(state1, state5);
        assert_ne!(state1, state6);
        assert_ne!(state1, state7);
    }

    #[test]
    fn test_mut_board() {
        let mut state = ChessState::from_fen("8/8/8/8/8/8/8/8 w - -");

        let b = state.bboard_mut(BBPiece::WPawn);
        assert_eq!(*b, 0u64);

        *b ^= 1;

        let b = state.bboard_mut(BBPiece::WPawn);
        assert_eq!(*b, 1u64);
    }

}
