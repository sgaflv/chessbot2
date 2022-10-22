use std::num::Wrapping;
use std::rc::Rc;

use crate::bboard::*;
use crate::debug::*;
use crate::game_setup::*;
use crate::magic::Magic;
use crate::piece_moves::*;
use crate::state::*;
use crate::state::{ChessState, BBPiece, Side};

pub struct MoveGenerator {
    move_provider: Rc<PieceMoveProvider>,
    magic: Magic,
}

impl MoveGenerator {
    /// Create new move generator
    pub fn new() -> MoveGenerator {
        let move_provider = Rc::new(PieceMoveProvider::new());

        MoveGenerator {
            move_provider: move_provider.clone(),
            magic: Magic::new(move_provider.clone()),
        }
    }

    fn fill_pawn_moves(
        &self,
        state: &mut ChessState,
        moves: &mut Vec<ChessMove>,
        move_from: BBoard,
        move_candidates: BBoard,
    ) {
        let mut move_candidates = move_candidates;

        let (this_ofs, other_ofs) = state.next_to_move.offsets();

        let (this_pawn, other_pawn) = if state.next_to_move == Side::White {
            (BBPiece::WPawn, BBPiece::BPawn)
        } else {
            (BBPiece::BPawn, BBPiece::WPawn)
        };

        let (this_en_passant, other_en_passant) = if state.next_to_move == Side::White {
            (BBPiece::WEnPassant, BBPiece::BEnPassant)
        } else {
            (BBPiece::BEnPassant, BBPiece::WEnPassant)
        };

        let (this_all, other_all) = if state.next_to_move == Side::White {
            (BBPiece::WAll, BBPiece::BAll)
        } else {
            (BBPiece::BAll, BBPiece::WAll)
        };

        let other_en_passant_bb = state.bboard_ofs(BBPiece::WEnPassant, other_ofs);

        let this_all_bb = state.bboard_ofs(BBPiece::WAll, this_ofs);
        let other_all_bb = state.bboard_ofs(BBPiece::WAll, other_ofs);
        let all_bb = this_all_bb | other_all_bb;

        while move_candidates > 0 {

            let move_to = move_candidates & (-Wrapping(move_candidates)).0;

            let mut new_move = ChessMove::new(state.next_to_move, move_from, move_to, None);

            let move_delta = move_from | (move_to & 0x00ffffffffffff00u64);

            new_move.add_delta(this_pawn, move_delta);
            new_move.add_delta(this_all, move_from | move_to);


            match move_to {
                WKING_SIDE_ROOK  => new_move.add_delta(BBPiece::WKCastle, state.bboard(BBPiece::WKCastle)),
                WQUEEN_SIDE_ROOK => new_move.add_delta(BBPiece::WQCastle, state.bboard(BBPiece::WQCastle)),
                BKING_SIDE_ROOK  => new_move.add_delta(BBPiece::BKCastle, state.bboard(BBPiece::BKCastle)),
                BQUEEN_SIDE_ROOK => new_move.add_delta(BBPiece::BQCastle, state.bboard(BBPiece::BQCastle)),
                _ => {}
            }

            // reset enemy en-passant
            if other_en_passant_bb > 0 {
                new_move.add_delta(other_en_passant, other_en_passant_bb);
            }

            // en-passant capture
            if move_to & other_en_passant_bb > 0 {
                let capture_target = (move_to << 8) | (move_to >> 8);
                let other_pawns = state.bboard_ofs(BBPiece::WPawn, other_ofs);

                new_move.add_delta(other_pawn, capture_target & other_pawns);
                new_move.add_delta(other_all, capture_target & other_pawns);
            }

            // create new en-passant
            if (move_from & 0x00ff00000000ff00u64) > 0u64 && (move_to & 0x000000ffff000000u64) > 0u64 {
                let this_en_passant_bb = ((move_to << 8) | (move_to >> 8)) & ((move_from << 8) | (move_from >> 8));

                if this_en_passant_bb & all_bb == 0 {
                    new_move.add_delta(this_en_passant, this_en_passant_bb);
                } else {
                    // en-passant move is blocked, hence continue without adding it
                    move_candidates ^= move_to;
                    continue;
                }
            }

            // pawn captures
            if other_all_bb & move_to > 0 {
                new_move.add_delta(state.piece_at(move_to), move_to);
                new_move.add_delta(other_all, move_to);
            }

            // promotions
            if move_to & 0xff000000000000ffu64 > 0 {

                let promotions = if this_ofs == 0 {
                    [BBPiece::WRook, BBPiece::WKnight, BBPiece::WBishop, BBPiece::WQueen]
                } else {
                    [BBPiece::BRook, BBPiece::BKnight, BBPiece::BBishop, BBPiece::BQueen]
                };

                for p in promotions.iter() {
                    
                    let mut new_move = new_move.clone();

                    new_move.add_delta(*p, move_to);

                    if !self.is_king_hit(state, &new_move) {
                        debug_assert!(state_is_sane(state, &new_move));
                        moves.push(new_move);
                    }
                }

                move_candidates ^= move_to;
                continue;
            }

            if !self.is_king_hit(state, &new_move) {
                debug_assert!(state_is_sane(state, &new_move));
                moves.push(new_move);
            }

            move_candidates ^= move_to;
        }
    }

    fn fill_rbqn_moves(
        &self,
        state: &mut ChessState,
        moves: &mut Vec<ChessMove>,
        piece: BBPiece,
        move_from: BBoard,
        move_candidates: BBoard,
    ) {
        let mut move_candidates = move_candidates;

        let (_this_ofs, other_ofs) = state.next_to_move.offsets();

        let (this_all, other_all) = if state.next_to_move == Side::White {
            (BBPiece::WAll, BBPiece::BAll)
        } else {
            (BBPiece::BAll, BBPiece::WAll)
        };


        let (_this_en_passant, other_en_passant) = if state.next_to_move == Side::White {
            (BBPiece::WEnPassant, BBPiece::BEnPassant)
        } else {
            (BBPiece::BEnPassant, BBPiece::WEnPassant)
        };
        
        let other_all_bb = state.bboard_ofs(BBPiece::WAll, other_ofs);
        let other_en_passant_bb = state.bboard_ofs(BBPiece::WEnPassant, other_ofs);

        while move_candidates > 0 {
            let move_to = move_candidates & (-Wrapping(move_candidates)).0;

            let move_delta = move_from | move_to;

            let mut new_move = ChessMove::new(state.next_to_move, move_from, move_to, None);

            // disable castle states
            match move_from {
                WKING_SIDE_ROOK  => new_move.add_delta(BBPiece::WKCastle, state.bboard(BBPiece::WKCastle)),
                WQUEEN_SIDE_ROOK => new_move.add_delta(BBPiece::WQCastle, state.bboard(BBPiece::WQCastle)),
                BKING_SIDE_ROOK  => new_move.add_delta(BBPiece::BKCastle, state.bboard(BBPiece::BKCastle)),
                BQUEEN_SIDE_ROOK => new_move.add_delta(BBPiece::BQCastle, state.bboard(BBPiece::BQCastle)),
                _ => {}
            }

            match move_to {
                WKING_SIDE_ROOK  => new_move.add_delta(BBPiece::WKCastle, state.bboard(BBPiece::WKCastle)),
                WQUEEN_SIDE_ROOK => new_move.add_delta(BBPiece::WQCastle, state.bboard(BBPiece::WQCastle)),
                BKING_SIDE_ROOK  => new_move.add_delta(BBPiece::BKCastle, state.bboard(BBPiece::BKCastle)),
                BQUEEN_SIDE_ROOK => new_move.add_delta(BBPiece::BQCastle, state.bboard(BBPiece::BQCastle)),
                _ => {}
            }

            if piece == BBPiece::WKing {
                new_move.add_delta(BBPiece::WKCastle, state.bboard(BBPiece::WKCastle));
                new_move.add_delta(BBPiece::WQCastle, state.bboard(BBPiece::WQCastle));
            }

            if piece == BBPiece::BKing {
                new_move.add_delta(BBPiece::BKCastle, state.bboard(BBPiece::BKCastle));
                new_move.add_delta(BBPiece::BQCastle, state.bboard(BBPiece::BQCastle));
            }

            new_move.add_delta(piece, move_delta);
            new_move.add_delta(this_all, move_delta);
            
            // reset enemy en-passant
            if other_en_passant_bb > 0 {
                new_move.add_delta(other_en_passant, other_en_passant_bb);
            }

            if  other_all_bb & move_to > 0 {
                new_move.add_delta(other_all, move_to);
                new_move.add_delta(state.piece_at(move_to), move_to);
            }

            if !self.is_king_hit(state, &new_move) {
                debug_assert!(state_is_sane(state, &new_move));
                moves.push(new_move);
            }

            move_candidates ^= move_to;
        }
    }

    fn fill_castle_moves(&self, state: &mut ChessState, moves: &mut Vec<ChessMove>) {
        
        let (king_castle, queen_castle) = state.castle_moves(state.next_to_move);

        if king_castle > 0u64 {
            
            self.fill_castle_move(state, moves, king_castle)
        }

        if queen_castle > 0u64 {

            self.fill_castle_move(state, moves, queen_castle)
        }
    }

    fn fill_castle_move(
        &self,
        state: &mut ChessState,
        moves: &mut Vec<ChessMove>,
        king_move: BBoard,
    ) {
        let all = state.bboard(BBPiece::WAll) | state.bboard(BBPiece::BAll);

        let (this_ofs, other_ofs) = state.next_to_move.offsets();

        let (this_king, this_rook, this_all, this_kcastle_piece, this_qcastle_piece) = 
            if state.next_to_move == Side::White {
                (BBPiece::WKing, BBPiece::WRook, BBPiece::WAll, BBPiece::WKCastle, BBPiece::WQCastle)
            } else {
                (BBPiece::BKing, BBPiece::BRook, BBPiece::BAll, BBPiece::BKCastle, BBPiece::BQCastle)
            };

        let other_en_passant = 
            if state.next_to_move == Side::White {
                BBPiece::BEnPassant
            } else {
                BBPiece::WEnPassant
            };

        let (king, _rook, empty, no_attack, king_move, rook_move) =
            ChessMove::castle_tuple_k_r_e_na_km_rm(king_move);

        if empty & all == 0 && !self.is_any_hit(state, no_attack, other_ofs) {
            let other_en_passant_bb = state.bboard_ofs(BBPiece::WEnPassant, other_ofs);

            let move_from = king;
            let move_to = king ^ king_move;
            let mut new_move = ChessMove::new(state.next_to_move, move_from, move_to, None);

            new_move.add_delta(this_king, king_move);
            new_move.add_delta(this_rook, rook_move);
            new_move.add_delta(this_all, rook_move | king_move);

            // disable castle states
            new_move.add_delta(this_kcastle_piece, state.bboard_ofs(BBPiece::WKCastle, this_ofs));
            new_move.add_delta(this_qcastle_piece, state.bboard_ofs(BBPiece::WQCastle, this_ofs));

            
            // reset enemy en-passant
            if other_en_passant_bb > 0 {
                new_move.add_delta(other_en_passant, other_en_passant_bb);
            }

            debug_assert!(state_is_sane(state, &new_move));

            moves.push(new_move);
        }
    }

    #[inline]
    pub fn is_king_hit(&self, state: &mut ChessState, chess_move: &ChessMove) -> bool {

        let (this_ofs, other_ofs) = state.next_to_move.offsets();

        state.do_move(chess_move);

        debug_assert!(state.bboard_ofs(BBPiece::WKing, this_ofs) > 0);
        let idx = state.bboard_ofs(BBPiece::WKing, this_ofs).trailing_zeros() as usize;

        let result = self.is_hit(state, idx, other_ofs);

        state.undo_move(chess_move);

        result
    }

     #[inline]
     fn is_any_hit(&self, state: &mut ChessState, check_board: BBoard, offset: usize) -> bool {

        let mut check_board = check_board;

        while check_board > 0 {
            let idx = last_bit(check_board).trailing_zeros() as usize;

            if self.is_hit(state, idx, offset) {
                return true;
            }

            check_board = remove_last_bit(check_board);
        }

        false
    }
    

    #[inline]
    fn is_hit(&self, state: &mut ChessState, idx: usize, offset: usize) -> bool {

        // use opposite pawn color to get source
        let pawns_capture = if offset == 0 {
            self.move_provider.black_pawn_capture
        } else {
            self.move_provider.white_pawn_capture
        };

        if state.bboard_ofs(BBPiece::WPawn, offset) & pawns_capture[idx] > 0 {
            return true;
        }

        if state.bboard_ofs(BBPiece::WKnight, offset) & self.move_provider.knight_move[idx] > 0 {
            return true;
        }

        let all = state.bboard(BBPiece::WAll) | state.bboard(BBPiece::BAll);

        let rook_moves = self.magic.get_rook_attack_bits(idx, all);
        let bishop_moves = self.magic.get_bishop_attack_bits(idx, all);

        if state.bboard_ofs(BBPiece::WRook, offset) & rook_moves > 0 {
            return true;
        }

        if state.bboard_ofs(BBPiece::WBishop, offset) & bishop_moves > 0 {
            return true;
        }

        if state.bboard_ofs(BBPiece::WQueen, offset) & (rook_moves | bishop_moves) > 0 {
            return true;
        }

        // hit by a king
        if state.bboard_ofs(BBPiece::WKing, offset) & self.move_provider.king_move[idx] > 0 {
            return true;
        }

        false
    }

    /// Function generates all possible moves from a given position, and fills them
    /// to the `moves` array. It returns the number of unique correct moves generated.
    pub fn generate_moves(&self, state: &mut ChessState, moves: &mut Vec<ChessMove>) {

        let (this_ofs, other_ofs) = state.next_to_move.offsets();

        let all_own_pieces_bb = state.bboard_ofs(BBPiece::WAll, this_ofs);
        let all_enemy_pieces_bb = state.bboard_ofs(BBPiece::WAll, other_ofs);

        let all_pieces = all_own_pieces_bb | all_enemy_pieces_bb;

        //
        let (this_king, this_rook, this_knight, this_bishop, this_queen) = 
        if state.next_to_move == Side::White {
            (BBPiece::WKing, BBPiece::WRook, BBPiece::WKnight, BBPiece::WBishop, BBPiece::WQueen)
        } else {
            (BBPiece::BKing, BBPiece::BRook, BBPiece::BKnight, BBPiece::BBishop, BBPiece::BQueen)
        };

        //////////////////////// pawns
        let mut pawns = state.bboard_ofs(BBPiece::WPawn, this_ofs);

        while pawns > 0 {
            let move_from = last_bit(pawns);

            let from_idx = move_from.trailing_zeros() as usize;

            let (pawn_moves, pawn_captures) = match state.next_to_move {
                Side::White => (
                    &self.move_provider.white_pawn_move,
                    &self.move_provider.white_pawn_capture,
                ),
                Side::Black => (
                    &self.move_provider.black_pawn_move,
                    &self.move_provider.black_pawn_capture,
                ),
            };

            let move_candidates = (pawn_moves[from_idx] & !all_own_pieces_bb & !all_enemy_pieces_bb)
                | (pawn_captures[from_idx]
                    & !all_own_pieces_bb
                    & (all_enemy_pieces_bb | state.bboard_ofs(BBPiece::WEnPassant, other_ofs)));

            self.fill_pawn_moves(state, moves, move_from, move_candidates);

            pawns = remove_last_bit(pawns);
        }

        ////////////////////////// rooks
        let mut rooks = state.bboard(this_rook);
        while rooks > 0 {
            let move_from = last_bit(rooks);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates =
                self.magic.get_rook_attack_bits(from_idx, all_pieces) & !all_own_pieces_bb;

            self.fill_rbqn_moves(state, moves, this_rook, move_from, move_candidates);

            rooks = remove_last_bit(rooks);
        }

        //////////////////////// knights
        let mut knights = state.bboard(this_knight);
        while knights > 0 {
            let move_from = last_bit(knights);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates = self.move_provider.knight_move[from_idx] & !all_own_pieces_bb;

            self.fill_rbqn_moves(state, moves, this_knight, move_from, move_candidates);

            knights = remove_last_bit(knights);
        }

        ///////////////////////// bishops
        let mut bishops = state.bboard(this_bishop);
        while bishops > 0 {
            let move_from = last_bit(bishops);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates =
                self.magic.get_bishop_attack_bits(from_idx, all_pieces) & !all_own_pieces_bb;

            self.fill_rbqn_moves(state, moves, this_bishop, move_from, move_candidates);

            bishops = remove_last_bit(bishops);
        }

        ///////////////////////// queens
        let mut queens = state.bboard(this_queen);
        while queens > 0 {
            let move_from = last_bit(queens);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates = (self.magic.get_rook_attack_bits(from_idx, all_pieces)
                | self.magic.get_bishop_attack_bits(from_idx, all_pieces))
                & !all_own_pieces_bb;
                
            self.fill_rbqn_moves(state, moves, this_queen, move_from, move_candidates);

            queens = remove_last_bit(queens);
        }

        //////////////////////// king
        let move_from = state.bboard(this_king);

        let from_idx = move_from.trailing_zeros() as usize;

        let move_candidates = self.move_provider.king_move[from_idx] & !all_own_pieces_bb;

        self.fill_rbqn_moves(state, moves, this_king, move_from, move_candidates);

        // add castle moves
        self.fill_castle_moves(state, moves);

    }
}

pub fn get_bb_move_text(
    piece: BBPiece,
    move_from: BBoard,
    move_to: BBoard,
) -> String {

    let mut move_str = String::new();

    move_str.push_str(piece.to_string().as_str());
    move_str.push_str(" ");
    move_str.push_str(bb_to_coord(move_from).as_str());
    move_str.push_str("-");
    move_str.push_str(bb_to_coord(move_to).as_str());

    return move_str;
}


pub fn get_move_text(chess_move: &ChessMove) -> String {

    get_bb_move_text(chess_move.get_piece(), chess_move.move_from, chess_move.move_to)

}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::debug::Demo;

    use super::*;

    fn perft_recursion(move_generator: &MoveGenerator, depth: u32, state: &mut ChessState) -> usize {
        if depth == 0 {
            return 1;
        }

        let mut result = 0;

        let mut new_moves: Vec<ChessMove> = Vec::with_capacity(20);

        move_generator.generate_moves(state, &mut new_moves);

        /*
        use chess::MoveGen;
        use chess::Board;
        use std::str::FromStr;

        let fen = state.to_fen();
        let board = Board::from_str(&fen);
        if board.is_err() {
            println!("fen parser error for board:{}", fen);
        }
        let board = board.unwrap();
        
        let iterable = MoveGen::new_legal(&board);
        if new_moves.len() != iterable.len() {
            println!("Count failure for position {}, expected {}, found {}", fen, iterable.len(), new_moves.len());
            
            panic!();
        }
*/
        for m in new_moves.iter() {
            state.do_move(m);
            result += perft_recursion(move_generator, depth - 1, state);
            state.undo_move(m);
        }

        result
    }

    fn perft_test(fen_state: &str, depth: u32, expected_move_count: usize) {

        let mut state = ChessState::from_fen(fen_state);
        println!("\nperft testing: {}", fen_state);
        println!("depth {}, board:", depth);
        let now = Instant::now();

        state.print();

        let move_generator = MoveGenerator::new();

        let move_count = perft_recursion(&move_generator, depth, &mut state);

        println!(
            "found:    {},\nexpected: {}",
            move_count, expected_move_count
        );
        println!("done in {} milliseconds\n", now.elapsed().as_millis());

        if move_count != expected_move_count {
            println!("perft failed for depth {} FEN {}", depth, fen_state);
        }

        assert_eq!(move_count, expected_move_count);
    }

    fn perft_tests(fen_state: &str, move_counts: &[u32]) {
        for (idx, move_count) in move_counts.iter().enumerate() {
            let depth = (idx + 1) as u32;
            let move_count = *move_count as usize;

            if move_count == 0 {
                continue;
            }

            perft_test(fen_state, depth, move_count);
        }
    }

    /* 
    #[test]
    fn test_chess() {
        let a = 3;

        let fen_state = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 1 0";
        let mut state = ChessState::from_fen(fen_state);
        state.demo();
        
        let move_generator = MoveGenerator::new();
        let mut new_moves: Vec<ChessMove> = Vec::with_capacity(20);

        move_generator.generate_moves(&mut state, &mut new_moves);

        use chess::MoveGen;
        use chess::Board;
        use std::str::FromStr;

        let fen = state.to_fen();
        let board = Board::from_str(&fen).expect("Invalid Position");
        
        let iterable = MoveGen::new_legal(&board);

        println!("Computed moves:");
        for (idx, cmove) in new_moves.iter().enumerate() {
            print!("move {} {}\n", idx,  cmove.to_string());

            /*if idx == 0 {
                cmove.demo();
            }*/
        }

        println!("Expected moves:");
        for (idx, cmove) in iterable.enumerate() {
            print!("move {} {}\n", idx,  cmove.to_string());
        }
        
    }
*/
    #[test]
    fn test_moves() {
        // https://www.chessprogramming.org/Perft_Results
        // initial position
        perft_tests("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &[20, 400, 8902, 197281, 4865609]);//, 119060324]);

        /*// position 2
        perft_tests(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
            &[48, 2039, 97862, 4085603, 193690690],
        );

        // position 3
        perft_tests(
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
            &[14, 191, 2812, 43238, 674624],
        );

        // position 4
        perft_tests(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            &[6, 264, 9467, 422333, 15833292],
        );
        // position 4m
        perft_tests(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
            &[6, 264, 9467, 422333, 15833292],
        );

        // position 5
        perft_tests(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            &[44, 1486, 62379, 2103487, 89941194],
        );

        // position 6
        perft_tests(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            &[46, 2079, 89890, 3894594, 164075551],
        );*/
    }

    #[test]
    fn test_some_moves() {
        let mut state = ChessState::from_fen("r3k1B1/8/3b4/p1pPNR1n/2P5/2N4P/PP5P/R2Q2K1 b q - 0 1");

        //        let state = GameState::from_fen("r3k2r/p1pp1pb1/bn2pnp1/3PN3/1q2P3/P1N2Q1p/2PBBPPP/R3K2R w KQkq -");
        //        let state = GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/1R2K2R w Qkq -");

        let generator = MoveGenerator::new();

        let mut moves: Vec<ChessMove> = Vec::new();

        generator.generate_moves(&mut state, &mut moves);

        for (idx, cmove) in moves.iter().enumerate() {
            print!("{} ", idx + 1);

            state.do_move(cmove);
            print_move_info(&mut state, cmove);

            debug_assert!(state_is_sane(&mut state, cmove));
        }
    }

    fn print_move_info(state: &mut ChessState, chess_move: &ChessMove) {

        let move_info = get_move_text(&chess_move);
        println!(
            "{}    ======================================================",
            move_info
        );

        state.demo();

        state.do_move(chess_move);
        state.demo();
        state.undo_move(chess_move);
    }
}

#[inline]
fn state_is_sane(state: &mut ChessState, chess_move: &ChessMove) -> bool {

    state.do_move(chess_move);

    let all = state.bboard(BBPiece::WAll) | state.bboard(BBPiece::BAll) ;

    let mut all2 = 0u64;

    for piece in BBPiece::get_pieces() {
        all2 ^= state.bboard(*piece);
    }

    let mut result = true;

    if (state.next_to_move == Side::White && state.bboard(BBPiece::WEnPassant) > 0 )
       || (state.next_to_move == Side::Black && state.bboard(BBPiece::BEnPassant) > 0) {
        println!("En-passant incompatible with the current player:");
        result = false;
    }

    if (state.bboard(BBPiece::BEnPassant) & 0xffff00ffffffffffu64 > 0)
       || (state.bboard(BBPiece::WEnPassant) & 0xffffffffff00ffffu64 > 0) {
        println!("Impossible en-passant position:");
        
        result = false;
    }

    if state.bboard(BBPiece::WKing).count_ones() != 1
       || state.bboard(BBPiece::BKing).count_ones() != 1 {
        println!("Impossible king state:");
     
        result = false;
    }

    if all != all2 {
        println!("Inconsistent all piece bitboard state:");
     
        result = false;
    }

    if !result {
        println!("State sanity failed!");
        state.demo();
        state.undo_move(chess_move);

        println!("State before move:");
        println!("{}", state.to_fen());
        println!("Last move info");
        chess_move.demo();
    
        return result;
    }

    state.undo_move(chess_move);


    result
}
