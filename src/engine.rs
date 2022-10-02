use crate::evaluator::evaluate_position;
use crate::game_setup::ChessMove;
use crate::move_generator::MoveGenerator;
use crate::state::{ChessState, Side};
use core::cmp;

pub struct ChessEngine {
    move_generator: MoveGenerator,
}

impl ChessEngine {
    pub fn new() -> ChessEngine {
        ChessEngine {
            move_generator: MoveGenerator::new(),
        }
    }

    pub fn min_max_search(&self, penalty: i32, depth: u32, alpha: i32, beta: i32, state: &ChessState) -> i32 {
        if depth == 0 {
            // just estimate the current position and return its score
            return evaluate_position(state);
        }

        let mut moves: Vec<ChessState> = Vec::new();
        self.move_generator.generate_moves(state, &mut moves);

        if moves.len() == 0 {
            let king_hit = self.move_generator.is_king_hit(state, state.next_to_move);

            return if king_hit {
                // checkmate
                state.next_to_move.value() * -100000
            } else {
                // draw
                0
            };
        }

        let mut alpha = alpha;
        let mut beta = beta;

        return if state.next_to_move == Side::White {

            for cur_state in moves.iter() {

                let score = self.min_max_search(penalty + 1, depth - 1, alpha, beta, cur_state);

                alpha = cmp::max(alpha, score);

                if alpha >= beta {
                    break
                }
            }

            alpha
        } else {

            for cur_state in moves.iter() {
                let score = self.min_max_search(penalty + 1, depth - 1, alpha, beta, cur_state);

                beta = cmp::min(beta, score);

                if alpha >= beta {
                    break
                }

            }

            beta
        }

    }

    pub fn find_best_move(&self, state: &ChessState) -> Option<ChessMove> {
        let mut moves: Vec<ChessState> = Vec::new();

        self.move_generator.generate_moves(state, &mut moves);

        if moves.len() == 0 {
            // checkmate or stalemate situation
            return None;
        }

        let mut best_score = 0;
        let mut best_index = 0usize;
        let (mut min, mut max) = (0, 0);

        for (idx, m) in moves.iter().enumerate() {
            let score = self.min_max_search(0, 4, i32::MIN, i32::MAX, m);

            if idx == 0 {
                min = score;
                max = score;
            }

            let is_new_best = idx == 0
                || state.next_to_move == Side::White && score > best_score
                || state.next_to_move == Side::Black && score < best_score;

            if is_new_best {
                best_score = score;
                best_index = idx;
            }
        }

        info!("min: {}, max: {}, best score: {}", min, max, best_score);
        let next_move = state.get_move(moves.get(best_index).unwrap());

        Some(next_move)
    }
}
