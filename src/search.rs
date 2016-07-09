//! Game Tree Search

use state::*;
use evaluation::*;
use std::cmp;

pub fn negamax( state: &mut State, depth: usize, mut alpha: i32, beta: i32 ) -> i32 {
    // TT should store alpha, beta, bestValue???
    // also return the principal variation
    // move ordering??

    if depth == 0 {
        static_eval( state )
    } else {
        let mut best_eval = -MATE_VALUE;
        let mut eval;

        let ( legal_moves, _status ) = state.node_info();
        let irs = state.ir_state();

        for mv in &legal_moves {
            state.make( mv );
            eval = -negamax( state, depth - 1, -beta, -alpha );
            state.unmake( mv, &irs );

            best_eval = cmp::max( best_eval, eval );
            alpha = cmp::max( alpha, eval );
            if beta <= alpha { break; } // Can also fail hard if you want
        }

        best_eval
    }
}
