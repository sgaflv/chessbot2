use std::fmt;
use std::fmt::Formatter;

use crate::bboard::*;
use crate::common::{add_bit, castle_tuple_k_r_e_na_km_rm, update_castles};
use crate::debug::Demo;
use crate::game_setup::{ChessCoord, ChessMove};
use crate::messaging::send_message;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Piece {
    King,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
}

pub static PIECES: [Piece; 6] = [
    Piece::King,
    Piece::Pawn,
    Piece::Rook,
    Piece::Knight,
    Piece::Bishop,
    Piece::Queen,
];

impl Piece {
    pub fn values() -> &'static [Piece; 6] {
        &PIECES
    }

    #[inline]
    pub fn idx(&self) -> usize {
        (*self) as usize
    }

    pub fn value(&self) -> i32 {
        match self {
            Piece::Pawn => 100,
            Piece::Rook => 500,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Queen => 900,
            Piece::King => 40000,
        }
    }

    pub fn to_string(&self, side: Side) -> String {
        match side {
            Side::White => match self {
                Piece::Pawn => "Pawn".to_string(),
                Piece::Rook => "Rook".to_string(),
                Piece::Knight => "Knight".to_string(),
                Piece::Bishop => "Bishop".to_string(),
                Piece::Queen => "Queen".to_string(),
                Piece::King => "King".to_string(),
            },
            Side::Black => match self {
                Piece::Pawn => "pawn".to_string(),
                Piece::Rook => "rook".to_string(),
                Piece::Knight => "knight".to_string(),
                Piece::Bishop => "bishop".to_string(),
                Piece::Queen => "queen".to_string(),
                Piece::King => "king".to_string(),
            },
        }
    }
    pub fn to_char(&self, side: Side) -> char {
        match side {
            Side::White => match self {
                Piece::Pawn => 'P',
                Piece::Rook => 'R',
                Piece::Knight => 'N',
                Piece::Bishop => 'B',
                Piece::Queen => 'Q',
                Piece::King => 'K',
            },
            Side::Black => match self {
                Piece::Pawn => 'p',
                Piece::Rook => 'r',
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Queen => 'q',
                Piece::King => 'k',
            },
        }
    }

    pub fn from_byte(c: &u8) -> (Piece, Side) {
        match *c {
            b'P' => (Piece::Pawn, Side::White),
            b'R' => (Piece::Rook, Side::White),
            b'N' => (Piece::Knight, Side::White),
            b'B' => (Piece::Bishop, Side::White),
            b'Q' => (Piece::Queen, Side::White),
            b'K' => (Piece::King, Side::White),
            b'p' => (Piece::Pawn, Side::Black),
            b'r' => (Piece::Rook, Side::Black),
            b'n' => (Piece::Knight, Side::Black),
            b'b' => (Piece::Bishop, Side::Black),
            b'q' => (Piece::Queen, Side::Black),
            b'k' => (Piece::King, Side::Black),
            _ => panic!(),
        }
    }
}

static INITIAL_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub fn idx(&self) -> i32 {
        *self as i32
    }

    pub fn value(&self) -> i32 {
        match self {
            Side::White => 1,
            Side::Black => -1,
        }
    }

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
pub struct SideState {
    pub king_side_castle: bool,
    pub queen_side_castle: bool,

    pub boards: [BBoard; 6],

    pub all: BBoard,
}

impl SideState {
    pub fn new() -> SideState {
        SideState {
            king_side_castle: false,
            queen_side_castle: false,
            boards: [0u64; 6],

            all: 0u64,
        }
    }

    pub fn update(&mut self) {
        self.all = 0u64;
        for b in self.boards.iter() {
            self.all |= *b;
        }
    }

    pub fn castle_state(&self) -> (bool, bool) {
        (self.king_side_castle, self.queen_side_castle)
    }

    pub fn set_castle_state(&mut self, state: (bool, bool)) {
        self.king_side_castle = state.0;
        self.queen_side_castle = state.1;
    }

    pub fn set_king_side_castle(&mut self, value: bool) {
        self.king_side_castle = value;
    }

    pub fn set_queen_side_castle(&mut self, value: bool) {
        self.queen_side_castle = value;
    }

    pub fn get_mut_board(&mut self, piece: Piece) -> &mut BBoard {
        &mut self.boards[piece.idx()]
    }

    pub fn get_board(&self, piece: Piece) -> BBoard {
        self.boards[piece.idx()]
    }

    pub fn add_bit(&mut self, piece: Piece, to_add: BBoard) {
        let board = self.get_mut_board(piece);
        *board |= to_add;
        self.all |= to_add;
    }

    pub fn remove_bit(&mut self, to_remove: BBoard) -> Option<Piece> {
        let inv = !to_remove;

        if self.all & to_remove == 0 {
            return None;
        }

        self.all &= inv;

        if self.get_board(Piece::Pawn) & to_remove > 0 {
            *self.get_mut_board(Piece::Pawn) &= inv;
            return Some(Piece::Pawn);
        }

        if self.get_board(Piece::Rook) & to_remove > 0 {
            *self.get_mut_board(Piece::Rook) &= inv;
            return Some(Piece::Rook);
        }

        if self.get_board(Piece::Knight) & to_remove > 0 {
            *self.get_mut_board(Piece::Knight) &= inv;
            return Some(Piece::Knight);
        }

        if self.get_board(Piece::Bishop) & to_remove > 0 {
            *self.get_mut_board(Piece::Bishop) &= inv;
            return Some(Piece::Bishop);
        }

        if self.get_board(Piece::Queen) & to_remove > 0 {
            *self.get_mut_board(Piece::Queen) &= inv;
            return Some(Piece::Queen);
        }

        if self.get_board(Piece::King) & to_remove > 0 {
            *self.get_mut_board(Piece::King) &= inv;
            return Some(Piece::King);
        }

        panic!();
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ChessState {
    pub next_to_move: Side,
    pub en_passant: BBoard, // en-passant field from the last move

    pub white_state: SideState,
    pub black_state: SideState,
    pub half_move_count: u32,
    pub full_move_count: u32,
}

impl ChessState {
    pub fn new_empty() -> ChessState {
        ChessState {
            next_to_move: Side::White,
            en_passant: 0,
            white_state: SideState::new(),
            black_state: SideState::new(),
            half_move_count: 0,
            full_move_count: 0,
        }
    }

    pub fn new_game() -> ChessState {
        ChessState::from_fen(INITIAL_BOARD)
    }

    pub fn init_next_move(&self) -> ChessState {
        let mut new_state = self.clone();

        new_state.next_to_move = self.next_to_move.opposite();
        new_state.en_passant = 0u64;
        if new_state.next_to_move == Side::White {
            new_state.full_move_count += 1;
        }
        new_state.half_move_count += 1;

        new_state
    }

    pub fn get_mut_side_state(&mut self, side: Side) -> &mut SideState {
        match side {
            Side::White => &mut self.white_state,
            Side::Black => &mut self.black_state,
        }
    }

    pub fn get_mut_sides_state(&mut self, side: Side) -> (&mut SideState, &mut SideState) {
        match side {
            Side::White => (&mut self.white_state, &mut self.black_state),
            Side::Black => (&mut self.black_state, &mut self.white_state),
        }
    }

    pub fn get_side_state(&self, side: Side) -> &SideState {
        match side {
            Side::White => &self.white_state,
            Side::Black => &self.black_state,
        }
    }

    pub fn get_mut_board(&mut self, coord: &(Piece, Side)) -> &mut BBoard {
        match coord.1 {
            Side::White => self.white_state.get_mut_board(coord.0),
            Side::Black => self.black_state.get_mut_board(coord.0),
        }
    }

    pub fn get_board(&self, coord: &(Piece, Side)) -> BBoard {
        match coord.1 {
            Side::White => self.white_state.get_board(coord.0),
            Side::Black => self.black_state.get_board(coord.0),
        }
    }

    pub fn castle_state(&self, side: Side) -> (bool, bool) {
        let side_state = self.get_side_state(side);
        (side_state.king_side_castle, side_state.queen_side_castle)
    }

    pub fn set_castle_state(&mut self, side: Side, state: (bool, bool)) {
        let side_state = self.get_mut_side_state(side);
        side_state.set_castle_state(state);
    }

    pub fn to_fen(&self) -> String {
        let mut board = ['.'; 64];

        for piece in Piece::values().iter() {
            for side in [Side::White, Side::Black].iter() {
                let coord = (*piece, *side);
                let bit_board = self.get_board(&coord);

                let c = piece.to_char(*side);

                for i in 0..64 {
                    if (bit_board >> i) & 1 == 1 {
                        board[i] = c;
                    }
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
        bb_to_coord(self.en_passant)
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
                    let board: &mut BBoard = state.get_mut_board(&Piece::from_byte(&c));

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

            state
                .get_mut_side_state(Side::White)
                .set_castle_state((wk, wq));
            state
                .get_mut_side_state(Side::Black)
                .set_castle_state((bk, bq));
        }

        if let Some(en_passant) = split.get(3) {
            if en_passant.len() == 2 {
                let bytes = en_passant.as_bytes();
                let x = bytes[0];
                let y = bytes[1];

                let idx = (y - b'1') as u64 * 8 + (x - b'a') as u64;
                state.en_passant = 1u64 << idx;
            }
        }

        if let Some(moves) = split.get(4) {
            state.half_move_count = moves.parse().unwrap();
        }

        if let Some(moves) = split.get(5) {
            state.full_move_count = moves.parse().unwrap();
        }

        state.black_state.update();
        state.white_state.update();

        state
    }

    pub fn get_move(&self, new_state: &ChessState) -> ChessMove {
        let next_to_move = self.next_to_move;

        let old_this = self.get_side_state(next_to_move);
        let old_other = self.get_side_state(next_to_move.opposite());
        let new_this = new_state.get_side_state(next_to_move);
        let new_other = new_state.get_side_state(next_to_move.opposite());

        let mut captured: Option<Piece> = None;
        let mut move_from: BBoard = 0;

        let mut move_to: BBoard = 0;
        let mut promote_to: Option<Piece> = None;

        let mut role: Option<Piece> = None;

        let mut is_castle = false;

        for p in Piece::values().iter() {
            let old_board = old_this.get_board(*p);
            let new_board = new_this.get_board(*p);
            let delta = old_board ^ new_board;

            if delta.count_ones() == 2 && role.is_some() {
                // this must be castling
                is_castle = true;
            }

            if delta.count_ones() > 0 {
                if old_board & delta > 0 {
                    move_from = old_board & delta;
                    role = Some(*p);
                }

                if new_board & delta > 0 {
                    move_to = new_board & delta;
                    promote_to = Some(*p);
                }
            }

            let old_other_board = old_other.get_board(*p);
            let new_other_board = new_other.get_board(*p);

            if (old_other_board ^ new_other_board).count_ones() == 1 {
                captured = Some(*p);
            }
        }

        if promote_to == role {
            promote_to = None;
        }

        if move_from == 0 || move_to == 0 {
            send_message("# Could not find the moving piece. Engine in panic!");

            println!("========found -> new ===============");
            self.demo();
            new_state.demo();

            println!("=======================");

            panic!();
        }

        if is_castle {
            let castle_side = if move_from > move_to {
                Piece::King
            } else {
                Piece::Queen
            };

            let (king_from, rook_from, _, _, king_move, rook_move) =
                castle_tuple_k_r_e_na_km_rm(next_to_move, castle_side);

            let king_to = king_from ^ king_move;
            let rook_to = rook_from ^ rook_move;

            let result = ChessMove::Castle {
                side: next_to_move,
                king_from: ChessCoord::from_bboard(king_from),
                king_to: ChessCoord::from_bboard(king_to),
                rook_from: ChessCoord::from_bboard(rook_from),
                rook_to: ChessCoord::from_bboard(rook_to),
            };

            return result;
        }

        let role = role.unwrap();

        if role == Piece::Pawn && move_to == self.en_passant {
            let captured = if self.en_passant < 1u64 << 32 {
                self.en_passant << 8u64
            } else {
                self.en_passant >> 8u64
            };

            return ChessMove::EnPassantCapture {
                side: next_to_move,
                move_from: ChessCoord::from_bboard(move_from),
                move_to: ChessCoord::from_bboard(move_to),
                captured: ChessCoord::from_bboard(captured),
            };
        }

        ChessMove::Normal {
            side: next_to_move,
            role,
            move_from: ChessCoord::from_bboard(move_from),
            move_to: ChessCoord::from_bboard(move_to),
            promote: promote_to,
            capture: captured,
        }
    }

    /// Mutate object state to a new value based on a move provided
    pub fn do_move(&mut self, chess_move: &ChessMove) {
        let mut new_state = self.init_next_move();

        match chess_move {
            ChessMove::Normal {
                side,
                role,
                move_from,
                move_to,
                promote,
                capture,
            } => {
                if *role == Piece::Pawn || capture.is_some() {
                    new_state.half_move_count = 0;
                }

                if *role == Piece::Pawn
                    && ((*move_from).y + 2 == (*move_to).y || (*move_from).y == (*move_to).y + 2)
                {
                    new_state.en_passant =
                        bb_coord((*move_from).x, ((*move_from).y + (*move_to).y) / 2);
                }

                let (mut side1, mut side2) = new_state.get_mut_sides_state(*side);

                let board: &mut BBoard = side1.get_mut_board(*role);

                *board ^= move_from.as_bboard();

                if let Some(captured) = capture {
                    let b_captured = side2.get_mut_board(*captured);
                    *b_captured &= !move_to.as_bboard();
                    side2.all &= !move_to.as_bboard();
                }

                match promote {
                    Some(new_piece) => {
                        let board = side1.get_mut_board(*new_piece);
                        *board ^= move_to.as_bboard();
                    }
                    None => {
                        *board ^= move_to.as_bboard();
                    }
                }

                side1.all &= !move_from.as_bboard();
                side1.all |= move_to.as_bboard();
            }
            ChessMove::Castle {
                side,
                king_from,
                king_to,
                rook_from,
                rook_to,
            } => {
                let (mut side1, _) = new_state.get_mut_sides_state(*side);

                let delta_king = king_from.as_bboard() | king_to.as_bboard();
                let delta_rook = rook_from.as_bboard() | rook_to.as_bboard();
                *side1.get_mut_board(Piece::King) ^= delta_king;
                *side1.get_mut_board(Piece::Rook) ^= delta_rook;
                side1.all ^= delta_king | delta_rook;
            }
            ChessMove::EnPassantCapture {
                side,
                move_from,
                move_to,
                captured,
            } => {
                new_state.half_move_count = 0;
                let (mut side1, mut side2) = new_state.get_mut_sides_state(*side);
                let pawns1: &mut BBoard = side1.get_mut_board(Piece::Pawn);
                let pawns2: &mut BBoard = side2.get_mut_board(Piece::Pawn);

                let delta_pawn = move_from.as_bboard() | move_to.as_bboard();
                let pawn_capture = captured.as_bboard();

                *pawns1 ^= delta_pawn;
                side1.all ^= delta_pawn;
                *pawns2 ^= pawn_capture;
                side2.all ^= pawn_capture;
            }
        }

        update_castles(&mut new_state, self.next_to_move);
        update_castles(&mut new_state, self.next_to_move.opposite());

        *self = new_state;
    }
}

impl Demo for ChessState {
    fn demo(&self) {
        println!("{}", self.to_fen());

        let mut result = ['.'; 64];

        for piece in Piece::values().iter() {
            for side in [Side::White, Side::Black].iter() {
                let coord = (*piece, *side);
                let board = self.get_board(&coord);

                let c = piece.to_char(*side);

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

        let (side1, _) = state.get_mut_sides_state(Side::White);

        let b = side1.get_mut_board(Piece::Pawn);
        assert_eq!(*b, 0u64);

        *b ^= 1;

        let b = side1.get_mut_board(Piece::Pawn);
        assert_eq!(*b, 1u64);
    }

    #[test]
    fn test_enum_index() {
        let a = Piece::Queen;
        println!("{}", a.idx());
    }
}
