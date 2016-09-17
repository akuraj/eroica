//! Game Tree Search

use state::*;
use evaluation::*;
use std::cmp;

pub fn quiescence( state: &mut State, mut alpha: i32, beta: i32 ) -> i32 {
    let ( legal_moves, status ) = state.node_info();

    if status == Status::Ongoing {
        let mut eval = static_eval( state );

        if beta <= eval {
            // Assuming that we are not in Zugzwang, the "Stand Pat" is a lower bound on the eval.
            // FIXME: Ideally, we might want to make a null_move and then return that score - given we might implement some sort of Tempo into the static_eval?
            // Failing soft
            eval
        } else {
            alpha = cmp::max( alpha, eval );
            let irs = state.ir_state();
            let tactical_moves = state.tactical_moves( &legal_moves );

            for mv in &tactical_moves {
                state.make( mv );
                eval = cmp::max( eval, -quiescence( state, -beta, -alpha ) );
                state.unmake( mv, &irs );

                alpha = cmp::max( alpha, eval );
                if beta <= alpha { break; } // Failing soft
            }

            eval
        }
    } else {
        if status == Status::Checkmate { -MATE_VALUE } else { DRAW_VALUE }
    }
}

pub fn negamax( state: &mut State, depth: usize, mut alpha: i32, beta: i32 ) -> i32 {
    // TT should store alpha, beta, bestValue???
    // also return the principal variation
    // move ordering??

    if depth == 0 {
        quiescence( state, alpha, beta )
    } else {
        let ( legal_moves, status ) = state.node_info();

        if status == Status::Ongoing {
            let irs = state.ir_state();
            let mut eval = -MATE_VALUE;

            for mv in &legal_moves {
                state.make( mv );
                eval = cmp::max( eval, -negamax( state, depth - 1, -beta, -alpha ) );
                state.unmake( mv, &irs );

                alpha = cmp::max( alpha, eval );
                if beta <= alpha { break; } // Failing soft
            }

            eval
        } else {
            if status == Status::Checkmate { -MATE_VALUE } else { DRAW_VALUE }
        }
    }
}
