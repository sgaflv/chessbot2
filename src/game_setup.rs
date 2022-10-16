
use std::num::Wrapping;
use std::result::Result;

use crate::bboard::BBoard;
use crate::engine::ChessEngine;
use crate::state::{ChessState, BBPiece, Side};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct ChessCoord {
    pub x: u8,
    pub y: u8,
}

impl ChessCoord {
    pub fn new(x: u8, y: u8) -> ChessCoord {
        ChessCoord { x, y }
    }

    pub fn from_string(coord: &[u8]) -> ChessCoord {
        let x = coord[0] - b'a';
        let y = coord[1] - b'1';

        ChessCoord { x, y }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(2);
        result.push((self.x + b'a') as char);
        result.push((self.y + b'1') as char);

        result
    }

    #[inline]
    pub fn as_bboard(&self) -> BBoard {
        1u64 << self.idx() as u64
    }

    #[inline]
    pub fn from_bboard(board: BBoard) -> ChessCoord {
        let idx = board.trailing_zeros();

        if idx > 63 {
            panic!();
        }

        ChessCoord {
            x: (idx % 8) as u8,
            y: (idx / 8) as u8,
        }
    }

    #[inline]
    pub fn idx(&self) -> usize {
        (self.y * 8 + self.x) as usize
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ChessMove {
    pub deltas: Vec<(BBPiece, BBoard)>,
    pub side: Side,
    pub move_from: BBoard,
    pub move_to: BBoard,
    pub promote: Option<BBPiece>
}

impl ChessMove {
    pub fn new(side: Side, move_from: BBoard, move_to: BBoard, promote: Option<BBPiece>) -> ChessMove {
        return ChessMove {
            deltas: Vec::new(),            
            side,
            move_from,
            move_to,
            promote,
        };
    }

    pub fn get_piece(&self) -> BBPiece {

        for (piece, delta) in self.deltas.iter() {
            if delta & self.move_from > 0 {
                return piece.clone();
            }
        }

        panic!("Unknown moving piece!");
    }

    pub fn add_delta(&mut self, piece: BBPiece, delta: BBoard) {
        if delta == 0u64 {
            return;
        }

        self.deltas.push((piece, delta));
    }

    /// return tuple with bitboards:
    /// king, rook, empty, no_attack, king_move, rook_move
    #[inline]
    pub fn castle_tuple_k_r_e_na_km_rm(
        king_move: BBoard
    ) -> (BBoard, BBoard, BBoard, BBoard, BBoard, BBoard) {

        let w_k = (
            0b00010000u64,
            0b10000000u64,
            0b01100000u64,
            0b01110000u64,
            0b01010000u64,
            0b10100000u64,
        );

        let w_q = (
            0b00010000u64, 
            0b00000001u64, 
            0b00001110u64, 
            0b00011100u64,
            0b00010100u64, 
            0b00001001u64,
        );

        let b_k = (
            0b00001000u64.reverse_bits(),
            0b00000001u64.reverse_bits(),
            0b00000110u64.reverse_bits(),
            0b00001110u64.reverse_bits(),
            0b00001010u64.reverse_bits(),
            0b00000101u64.reverse_bits(),
        );

        let b_q = (
            0b00001000u64.reverse_bits(),
            0b10000000u64.reverse_bits(),
            0b01110000u64.reverse_bits(),
            0b00111000u64.reverse_bits(),
            0b00101000u64.reverse_bits(),
            0b10010000u64.reverse_bits(),
        );

        let result = match king_move {
            0b01010000u64 => w_k,
            0b00010100u64 => w_q,
            0b0000101000000000000000000000000000000000000000000000000000000000u64 => b_k,
            0b0010100000000000000000000000000000000000000000000000000000000000u64 => b_q,
            _ => panic!(),
        };

        result
    }



    pub fn parse(move_str: &str, curr_state: &ChessState) -> Result<ChessMove, String> {
        let from_str = &move_str.as_bytes()[0..2];
        let to_str = &move_str.as_bytes()[2..4];

        let promo_string = &move_str.as_bytes().get(4);

        let promote = 
            if let Some(promo_char) = promo_string {
                Some(BBPiece::from_byte(promo_char))
            } else {
                None
            };

        let move_from = ChessCoord::from_string(from_str);
        let move_to = ChessCoord::from_string(to_str);

        let side = curr_state.next_to_move;

        let move_from_b = move_from.as_bboard();
        let move_to_b = move_to.as_bboard();

        let mut role: Option<BBPiece> = None;
        let mut capture: Option<BBPiece> = None;

        let mut is_castle = false;

        let mut result = ChessMove::new(side, move_from_b, move_to_b, promote);

        result.promote = promote;

        for p in BBPiece::get_pieces() {

            let board_from = move_from_b & curr_state.bboard(p);

            let board_to = move_to_b & curr_state.bboard(p);

            if (board_from > 0) && (p.get_side() == side) {
                role = Some(p);
                
                result.add_delta(p, move_from_b ^ move_to_b);

                if (p == BBPiece::WKing || p == BBPiece::BKing)
                    && (Wrapping(move_to_b) << 2 == Wrapping(move_from_b)
                        || Wrapping(move_to_b) >> 2 == Wrapping(move_from_b))
                {
                    is_castle = true;
                    
                }
            }

            if board_to > 0 {
                capture = Some(p);
                
                result.add_delta(p, move_to_b);
            }
        }

        if let Some(cap) = capture {
            if cap.get_side() == side {
                role = None;
            }
        }

        if role.is_none() {
            let msg = format!("failed to parse {:?} on board: {:?}", move_str, curr_state);
            return Err(msg);
        }

        let role = role.unwrap();

        if role == BBPiece::WPawn && move_to_b == curr_state.bboard(BBPiece::BEnPassant) {
            result.add_delta(BBPiece::BPawn, move_to_b << 8u64);

            return Ok(result);
        }

        if role == BBPiece::BPawn && move_to_b == curr_state.bboard(BBPiece::WEnPassant) {
            result.add_delta(BBPiece::WPawn, move_to_b >> 8u64);

            return Ok(result);
        }

        if is_castle {

            let (_, _, _, _, _, rook_move) =
                Self::castle_tuple_k_r_e_na_km_rm(move_from_b | move_to_b);
                
            if side == Side::White {
                result.add_delta(BBPiece::WRook, rook_move);
            } else {
                result.add_delta(BBPiece::BRook, rook_move);
            }
            
        }

        Ok(result)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(4);

        result.push_str(self.move_from.to_string().as_str());
        result.push_str(self.move_from.to_string().as_str());
        
        if let Some(promoted) = self.promote {
            result.push(promoted.to_char());
        }

        result
    }
}

pub struct GameSetup {
    pub xboard: bool,
    pub pondering: bool,

    pub computer_player: [bool; 2],

    pub time: i32,
    pub otime: i32,
    pub moves_left: i32,

    pub forced: bool,

    pub game_state: ChessState,

    pub engine: ChessEngine,
}

impl GameSetup {
    pub fn new() -> GameSetup {
        let result = GameSetup {
            xboard: false,
            pondering: false,
            computer_player: [false, false],
            time: 0,
            otime: 0,
            moves_left: 0,
            forced: false,
            game_state: ChessState::new_game(),
            engine: ChessEngine::new(),
        };

        result
    }
}
