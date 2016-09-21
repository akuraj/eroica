//! Game Tree Search

use state::*;
use evaluation::*;
use std::cmp;
use std::collections::VecDeque;

pub struct Variation {
    pub eval: i32,
    pub move_list: VecDeque<Move>,
}

impl Variation {
    pub fn terminal( eval: i32 ) -> Self {
        Variation { eval: eval,
                    move_list: VecDeque::new(), }
    }

    pub fn max_assign( &mut self, mv: &Move, var: Variation ) {
        if self.eval < -( var.eval ) {
            self.eval = -var.eval;
            self.move_list = var.move_list;
            self.move_list.push_front( *mv );
        }
    }
}

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

pub fn negamax( state: &mut State, depth: usize, mut alpha: i32, beta: i32 ) -> Variation {
    let ( legal_moves, status ) = state.node_info();

    if status == Status::Ongoing {
        if depth == 0 {
            Variation::terminal( quiescence( state, alpha, beta ) )
        } else {
            let irs = state.ir_state();
            let mut var = Variation::terminal( -MATE_VALUE );

            for mv in &legal_moves {
                state.make( mv );
                var.max_assign( mv, negamax( state, depth - 1, -beta, -alpha ) );
                state.unmake( mv, &irs );

                alpha = cmp::max( alpha, var.eval );
                if beta <= alpha { break; } // Failing soft
            }

            var
        }
    } else {
        if status == Status::Checkmate { Variation::terminal( -MATE_VALUE ) } else { Variation::terminal( DRAW_VALUE ) }
    }
}
