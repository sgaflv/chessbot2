use std::collections::HashMap;
use std::num::Wrapping;
use std::result::Result;

use crate::bboard::BBoard;
use crate::common::castle_tuple_k_r_e_na_km_rm;
use crate::engine::ChessEngine;
use crate::state::{ChessState, Piece, Side};

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

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ChessMove {
    Normal {
        side: Side,
        role: Piece,
        move_from: ChessCoord,
        move_to: ChessCoord,
        promote: Option<Piece>,
        capture: Option<Piece>,
    },
    Castle {
        side: Side,
        king_from: ChessCoord,
        king_to: ChessCoord,
        rook_from: ChessCoord,
        rook_to: ChessCoord,
    },
    EnPassantCapture {
        side: Side,
        move_from: ChessCoord,
        move_to: ChessCoord,
        captured: ChessCoord,
    },
}

impl ChessMove {
    pub fn parse(move_str: &str, curr_state: &ChessState) -> Result<ChessMove, String> {
        let from_str = &move_str.as_bytes()[0..2];
        let to_str = &move_str.as_bytes()[2..4];
        let promo = &move_str.as_bytes().get(4);

        let promote = promo.map(|i| {
            let (p, _) = Piece::from_byte(i);
            p
        });

        let move_from = ChessCoord::from_string(from_str);
        let move_to = ChessCoord::from_string(to_str);
        let side = curr_state.next_to_move;

        let move_from_b = move_from.as_bboard();
        let move_to_b = move_to.as_bboard();

        let this_side_state = curr_state.get_side_state(side);
        let other_side_state = curr_state.get_side_state(side.opposite());
        let mut role: Option<Piece> = None;
        let mut capture: Option<Piece> = None;

        let mut is_castle = false;

        for p in Piece::values().iter() {
            let own_board = move_from_b & this_side_state.get_board(*p);

            if own_board > 0 {
                role = Some(*p);

                if *p == Piece::King
                    && (Wrapping(move_to_b) << 2 == Wrapping(move_from_b)
                        || Wrapping(move_to_b) >> 2 == Wrapping(move_from_b))
                {
                    is_castle = true;
                }
            }

            let other_board = move_to_b & other_side_state.get_board(*p);
            if other_board > 0 {
                capture = Some(*p);
            }
        }

        if role.is_none() {
            let msg = format!("failed to parse {:?} on board: {:?}", move_str, curr_state);
            return Err(msg);
        }

        let role = role.unwrap();

        if role == Piece::Pawn && move_to_b == curr_state.en_passant {
            let captured = if curr_state.en_passant < 1u64 << 32 {
                curr_state.en_passant << 8u64
            } else {
                curr_state.en_passant >> 8u64
            };

            return Ok(ChessMove::EnPassantCapture {
                side,
                move_from,
                move_to,
                captured: ChessCoord::from_bboard(captured),
            });
        }

        if is_castle {
            let castle_side = if move_from_b < move_to_b {
                Piece::King
            } else {
                Piece::Queen
            };

            let (king_from, rook_from, _, _, king_move, rook_move) =
                castle_tuple_k_r_e_na_km_rm(side, castle_side);

            let king_to = king_from ^ king_move;
            let rook_to = rook_from ^ rook_move;

            return Ok(ChessMove::Castle {
                side,
                king_from: ChessCoord::from_bboard(king_from),
                king_to: ChessCoord::from_bboard(king_to),
                rook_from: ChessCoord::from_bboard(rook_from),
                rook_to: ChessCoord::from_bboard(rook_to),
            });
        }

        Ok(ChessMove::Normal {
            side,
            role,
            move_from,
            move_to,
            promote,
            capture,
        })
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(4);

        match self {
            ChessMove::Normal {
                move_from: from,
                move_to: to,
                promote,
                ..
            } => {
                result.push_str(from.to_string().as_str());
                result.push_str(to.to_string().as_str());
                if let Some(promoted) = promote {
                    result.push(promoted.to_char(Side::Black));
                }
            }
            ChessMove::Castle {
                king_from: from,
                king_to: to,
                ..
            } => {
                result.push_str(from.to_string().as_str());
                result.push_str(to.to_string().as_str());
            }
            ChessMove::EnPassantCapture {
                move_from: from,
                move_to: to,
                ..
            } => {
                result.push_str(from.to_string().as_str());
                result.push_str(to.to_string().as_str());
            }
        }

        result
    }
}

pub struct GameSetup {
    pub xboard: bool,
    pub pondering: bool,

    pub computer_player: HashMap<Side, bool>,

    pub time: i32,
    pub otime: i32,
    pub moves_left: i32,

    pub forced: bool,

    pub game_state: ChessState,

    pub engine: ChessEngine,
}

impl GameSetup {
    pub fn new() -> GameSetup {
        let mut result = GameSetup {
            xboard: false,
            pondering: false,
            computer_player: HashMap::new(),
            time: 0,
            otime: 0,
            moves_left: 0,
            forced: false,
            game_state: ChessState::new_game(),
            engine: ChessEngine::new(),
        };

        result.computer_player.insert(Side::White, false);
        result.computer_player.insert(Side::Black, false);

        result
    }
}
