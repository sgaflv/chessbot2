use std::num::Wrapping;
use std::rc::Rc;

use crate::bboard::{bb_to_coord, BBoard, last_bit, remove_last_bit};
use crate::common::{castle_tuple_k_r_e_na_km_rm, update_castles};
use crate::debug::*;
use crate::game_setup::ChessMove;
use crate::magic::Magic;
use crate::piece_moves::*;
use crate::state::{ChessState, Piece, Side};

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
        old_state: &ChessState,
        moves: &mut Vec<ChessState>,
        move_from: BBoard,
        move_candidates: BBoard,
    ) {
        let mut move_candidates = move_candidates;
        let next_to_move = old_state.next_to_move;

        while move_candidates > 0 {
            let move_to = move_candidates & (-Wrapping(move_candidates)).0;

            let move_delta = move_from | move_to;

            let mut new_state: ChessState = (*old_state).init_next_move();
            new_state.half_move_count = 0;

            let mut en_passant = 0u64;

            let (side_state, other_state) = new_state.get_mut_sides_state(next_to_move);

            side_state.boards[Piece::Pawn.idx()] ^= move_delta;
            side_state.all ^= move_delta;

            // en-passant capture
            if move_to & (*old_state).en_passant > 0 {
                if (*old_state).en_passant < (1u64 << 32) {
                    other_state.remove_bit(move_to << 8);
                } else {
                    other_state.remove_bit(move_to >> 8);
                }
            }

            if move_delta & (Wrapping(move_delta) << 16).0 > 0 {
                let mut enp = Wrapping(move_delta) << 8;
                enp = enp & -enp;
                en_passant = enp.0;
            }

            let removed = other_state.remove_bit(move_to);

            // promotions
            if move_to & 0xff000000000000ffu64 > 0 {
                side_state.boards[Piece::Pawn.idx()] ^= move_to;

                for p in [Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen].iter() {
                    let mut new_state = new_state.clone();

                    if let Some(piece) = removed {
                        if piece == Piece::Rook {
                            update_castles(&mut new_state, next_to_move.opposite());
                        }
                    }

                    let board = new_state.get_mut_board(&(*p, next_to_move));
                    *(board) ^= move_to;

                    if !self.is_king_hit(&new_state, next_to_move) {
                        update_castles(&mut new_state, next_to_move);

                        debug_assert!(state_is_sane(old_state, &new_state));
                        moves.push(new_state);
                    }
                }

                move_candidates ^= move_to;
                continue;
            }

            if en_passant & (other_state.all | side_state.all) == 0 {
                new_state.en_passant = en_passant;
                if !self.is_king_hit(&new_state, next_to_move) {
                    update_castles(&mut new_state, next_to_move);

                    debug_assert!(state_is_sane(old_state, &new_state));
                    moves.push(new_state);
                }
            }

            move_candidates ^= move_to;
        }
    }

    fn fill_rbqn_moves(
        &self,
        state: &ChessState,
        moves: &mut Vec<ChessState>,
        piece: Piece,
        move_from: BBoard,
        move_candidates: BBoard,
    ) {
        let mut move_candidates = move_candidates;
        let next_to_move = state.next_to_move;

        while move_candidates > 0 {
            let move_to = move_candidates & (-Wrapping(move_candidates)).0;

            let move_delta = move_from | move_to;

            let mut new_state: ChessState = (*state).init_next_move();

            let (side_state, other_state) = new_state.get_mut_sides_state(next_to_move);

            side_state.boards[piece.idx()] ^= move_delta;
            side_state.all ^= move_delta;

            if let Some(piece) = other_state.remove_bit(move_to) {
                new_state.half_move_count = 0;

                if piece == Piece::Rook {
                    update_castles(&mut new_state, next_to_move.opposite());
                }
            }

            if !self.is_king_hit(&new_state, next_to_move) {
                update_castles(&mut new_state, next_to_move);

                debug_assert!(state_is_sane(&state, &new_state));
                moves.push(new_state);
            }

            move_candidates ^= move_to;
        }
    }

    fn fill_castle_moves(&self, state: &ChessState, moves: &mut Vec<ChessState>, side: Side) {
        let side_state = state.get_side_state(side);

        if side_state.king_side_castle {
            self.fill_castle_move(state, moves, side, Piece::King)
        }

        if side_state.queen_side_castle {
            self.fill_castle_move(state, moves, side, Piece::Queen)
        }
    }

    fn fill_castle_move(
        &self,
        state: &ChessState,
        moves: &mut Vec<ChessState>,
        side: Side,
        castle_type: Piece,
    ) {
        let all = state.white_state.all | state.black_state.all;

        let (_, _, empty, no_attack, king_move, rook_move) =
            castle_tuple_k_r_e_na_km_rm(side, castle_type);

        if empty & all == 0 && !self.is_any_hit(state, side, no_attack) {
            let mut new_state = state.init_next_move();

            let side_state = new_state.get_mut_side_state(side);

            side_state.boards[Piece::King.idx()] ^= king_move;
            side_state.boards[Piece::Rook.idx()] ^= rook_move;
            side_state.queen_side_castle = false;
            side_state.king_side_castle = false;
            side_state.update();

            debug_assert!(state_is_sane(state, &new_state));
            moves.push(new_state);
        }
    }

    #[inline]
    pub fn is_king_hit(&self, state: &ChessState, side: Side) -> bool {
        debug_assert!(state.get_side_state(side).boards[Piece::King.idx()] > 0);

        let idx = state.get_side_state(side).boards[Piece::King.idx()].trailing_zeros() as usize;

        self.is_hit(state, side, idx)
    }

    #[inline]
    fn is_any_hit(&self, state: &ChessState, side: Side, check_board: BBoard) -> bool {
        let mut check_board = check_board;

        while check_board > 0 {
            let idx = last_bit(check_board).trailing_zeros() as usize;

            if self.is_hit(state, side, idx) {
                return true;
            }

            check_board = remove_last_bit(check_board);
        }

        false
    }

    #[inline]
    fn is_hit(&self, state: &ChessState, side: Side, idx: usize) -> bool {
        let enemy = side.opposite();

        let enemy_state = state.get_side_state(enemy);

        // use opposite pawn color to get source
        let pawns_capture = if side == Side::White {
            self.move_provider.white_pawn_capture
        } else {
            self.move_provider.black_pawn_capture
        };

        if enemy_state.boards[Piece::Pawn.idx()] & pawns_capture[idx] > 0 {
            return true;
        }

        if enemy_state.boards[Piece::Knight.idx()] & self.move_provider.knight_move[idx] > 0 {
            return true;
        }

        // hit by a king
        if enemy_state.boards[Piece::King.idx()] & self.move_provider.king_move[idx] > 0 {
            return true;
        }

        let all = state.white_state.all | state.black_state.all;
        let rook_moves = self.magic.get_rook_attack_bits(idx, all);
        let bishop_moves = self.magic.get_bishop_attack_bits(idx, all);

        if enemy_state.boards[Piece::Rook.idx()] & rook_moves > 0 {
            return true;
        }

        if enemy_state.boards[Piece::Bishop.idx()] & bishop_moves > 0 {
            return true;
        }

        if enemy_state.boards[Piece::Queen.idx()] & (rook_moves | bishop_moves) > 0 {
            return true;
        }

        false
    }

    /// Function generates all possible moves and returns them as a vector
    /// of ChessMove objects
    pub fn generate_moves2(&self, state: &ChessState) -> Vec<ChessMove> {
        let mut result = Vec::new();
        let mut move_states: Vec<ChessState> = Vec::new();

        self.generate_moves(state, &mut move_states);

        for s in move_states.iter() {
            let chess_move = state.get_move(s);
            result.push(chess_move);
        }

        result
    }

    /// Function generates all possible moves from a given position, and fills them
    /// to the `moves` array. It returns the number of unique correct moves generated.
    pub fn generate_moves(&self, state: &ChessState, moves: &mut Vec<ChessState>) {
        let next_to_move = state.next_to_move;

        let own_side_state = state.get_side_state(next_to_move);
        let opposite_side_state = state.get_side_state(next_to_move.opposite());

        let all_own_pieces = own_side_state.all;
        let all_enemy_pieces = opposite_side_state.all;
        let all_pieces = all_own_pieces | all_enemy_pieces;

        //////////////////////// pawns
        let mut pawns = own_side_state.boards[Piece::Pawn.idx()];
        while pawns > 0 {
            let move_from = last_bit(pawns);

            let from_idx = move_from.trailing_zeros() as usize;

            let (pawn_moves, pawn_captures) = match next_to_move {
                Side::White => (
                    &self.move_provider.white_pawn_move,
                    &self.move_provider.white_pawn_capture,
                ),
                Side::Black => (
                    &self.move_provider.black_pawn_move,
                    &self.move_provider.black_pawn_capture,
                ),
            };

            let move_candidates = (pawn_moves[from_idx] & !all_own_pieces & !all_enemy_pieces)
                | (pawn_captures[from_idx]
                    & !all_own_pieces
                    & (all_enemy_pieces | state.en_passant));

            self.fill_pawn_moves(state, moves, move_from, move_candidates);

            pawns = remove_last_bit(pawns);
        }

        ////////////////////////// rooks
        let mut rooks = own_side_state.boards[Piece::Rook.idx()];
        while rooks > 0 {
            let move_from = last_bit(rooks);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates =
                self.magic.get_rook_attack_bits(from_idx, all_pieces) & !all_own_pieces;

            self.fill_rbqn_moves(state, moves, Piece::Rook, move_from, move_candidates);

            rooks = remove_last_bit(rooks);
        }

        ///////////////////////// bishops
        let mut bishops = own_side_state.boards[Piece::Bishop.idx()];
        while bishops > 0 {
            let move_from = last_bit(bishops);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates =
                self.magic.get_bishop_attack_bits(from_idx, all_pieces) & !all_own_pieces;

            self.fill_rbqn_moves(state, moves, Piece::Bishop, move_from, move_candidates);

            bishops = remove_last_bit(bishops);
        }

        ///////////////////////// queens
        let mut queens = own_side_state.boards[Piece::Queen.idx()];
        while queens > 0 {
            let move_from = last_bit(queens);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates = (self.magic.get_rook_attack_bits(from_idx, all_pieces)
                | self.magic.get_bishop_attack_bits(from_idx, all_pieces))
                & !all_own_pieces;

            self.fill_rbqn_moves(state, moves, Piece::Queen, move_from, move_candidates);

            queens = remove_last_bit(queens);
        }

        //////////////////////// knights
        let mut knights = own_side_state.boards[Piece::Knight.idx()];
        while knights > 0 {
            let move_from = last_bit(knights);

            let from_idx = move_from.trailing_zeros() as usize;

            let move_candidates = self.move_provider.knight_move[from_idx] & !all_own_pieces;

            self.fill_rbqn_moves(state, moves, Piece::Knight, move_from, move_candidates);

            knights = remove_last_bit(knights);
        }

        //////////////////////// king
        let move_from = own_side_state.boards[Piece::King.idx()];

        let from_idx = move_from.trailing_zeros() as usize;

        let move_candidates = self.move_provider.king_move[from_idx] & !all_own_pieces;

        self.fill_rbqn_moves(state, moves, Piece::King, move_from, move_candidates);
        // add castle moves
        self.fill_castle_moves(state, moves, next_to_move);
    }
}

pub fn get_bb_move_text(
    piece: Piece,
    side: Side,
    state_from: BBoard,
    state_to: BBoard,
) -> Option<String> {
    if state_from == state_to {
        return None;
    }

    let diff = state_from ^ state_to;

    let ones = diff.count_ones();
    let mut move_str = String::new();
    if ones == 2 {
        let from = state_from & diff;
        let to = state_to & diff;
        move_str.push_str(piece.to_string(side).as_str());
        move_str.push_str(" ");
        move_str.push_str(bb_to_coord(from).as_str());
        move_str.push_str("-");
        move_str.push_str(bb_to_coord(to).as_str());
    } else if ones == 1 {
        move_str.push_str(piece.to_string(side).as_str());
        move_str.push_str(" ");
        move_str.push_str(bb_to_coord(diff).as_str());
        move_str.push_str(".");
    }

    return Some(move_str);
}

pub fn get_move_text(state_from: &ChessState, state_to: &ChessState) -> String {
    let side = state_from.next_to_move;
    debug_assert_eq!(side.opposite(), state_to.next_to_move);

    let mut result = String::new();

    let side_from = state_from.get_side_state(side);
    let side_to = state_to.get_side_state(side);

    for p in Piece::values().iter() {
        if let Some(m) = get_bb_move_text(*p, side, side_from.get_board(*p), side_to.get_board(*p))
        {
            result.push_str(m.as_str());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::debug::Demo;

    use super::*;

    fn perft_recursion(move_generator: &MoveGenerator, depth: u32, state: &ChessState) -> usize {
        if depth == 0 {
            return 1;
        }

        let mut result = 0;

        let mut new_moves: Vec<ChessState> = Vec::new();

        move_generator.generate_moves(state, &mut new_moves);

        let chess_moves = move_generator.generate_moves2(state);

        for (idx, cm) in chess_moves.iter().enumerate() {
            let initial = new_moves.get(idx).unwrap();
            let mut alternative = state.clone();
            alternative.do_move(cm);

            if !initial.eq(&alternative) {
                println!("Failed match for move: {:?}", cm);
                println!("Initial state:");
                state.demo();

                println!("Expected state: {:?}", initial);
                println!("Found state:    {:?}", alternative);
                panic!();
            }
        }

        for m in new_moves.iter() {
            result += perft_recursion(move_generator, depth - 1, m);
        }

        result
    }

    fn perft_test(fen_state: &str, depth: u32, expected_move_count: usize) {
        let state = ChessState::from_fen(fen_state);
        println!("\nperft testing: {}", fen_state);
        println!("depth {}, board:", depth);
        let now = Instant::now();

        state.demo();

        let move_generator = MoveGenerator::new();

        let move_count = perft_recursion(&move_generator, depth, &state);

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

    #[test]
    fn test_moves() {
        // https://www.chessprogramming.org/Perft_Results
        // initial position
        //perft_tests("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &[20, 400, 8902, 197281, 4865609, 119060324]);

        // position 2
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
        );
    }

    #[test]
    fn test_some_moves() {
        let state = ChessState::from_fen("r3k1B1/8/3b4/p1pPNR1n/2P5/2N4P/PP5P/R2Q2K1 b q - 0 1");

        //        let state = GameState::from_fen("r3k2r/p1pp1pb1/bn2pnp1/3PN3/1q2P3/P1N2Q1p/2PBBPPP/R3K2R w KQkq -");
        //        let state = GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/1R2K2R w Qkq -");

        let generator = MoveGenerator::new();

        let mut moves: Vec<ChessState> = Vec::new();

        generator.generate_moves(&state, &mut moves);

        for (idx, st) in moves.iter().enumerate() {
            print!("{} ", idx + 1);
            print_move_info(&state, st);

            debug_assert!(state_is_sane(&state, &st));
        }
    }

    fn print_move_info(from_state: &ChessState, to_state: &ChessState) {
        let move_info = get_move_text(&from_state, &to_state);
        println!(
            "{}    ======================================================",
            move_info
        );
        from_state.demo();
        to_state.demo();
    }
}

#[inline]
fn state_is_sane(old_state: &ChessState, state: &ChessState) -> bool {
    let all = state.white_state.all | state.black_state.all | state.en_passant;

    let boards = [
        state.white_state.get_board(Piece::Pawn),
        state.white_state.get_board(Piece::Rook),
        state.white_state.get_board(Piece::Knight),
        state.white_state.get_board(Piece::Bishop),
        state.white_state.get_board(Piece::Queen),
        state.white_state.get_board(Piece::King),
        state.en_passant,
        state.black_state.get_board(Piece::Pawn),
        state.black_state.get_board(Piece::Rook),
        state.black_state.get_board(Piece::Knight),
        state.black_state.get_board(Piece::Bishop),
        state.black_state.get_board(Piece::Queen),
        state.black_state.get_board(Piece::King),
    ];

    if state.en_passant > 0
        && ((state.next_to_move == Side::White && state.en_passant < 0x1u64 << 32)
            || (state.next_to_move == Side::Black && state.en_passant > 0x1u64 << 32))
    {
        println!("Impossible en-passant state:");
        state.demo();

        println!("old state:");
        old_state.demo();

        return false;
    }

    let mut all2 = 0u64;

    for x in boards.iter() {
        all2 ^= *x;
    }

    if state.white_state.boards[Piece::King.idx()].count_ones() != 1
        || state.black_state.boards[Piece::King.idx()].count_ones() != 1
    {
        println!("Impossible king state:");
        state.demo();

        println!("old state:");
        old_state.demo();

        return false;
    }

    if all != all2 {
        println!("Incoherent piece bitboard state:");
        state.demo();

        println!("old state:");
        old_state.demo();
        return false;
    }

    true
}
