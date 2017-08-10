//! Game Tree Search

use state::*;
use consts::*;
use std::cmp;
use std::collections::VecDeque;
use hashtables::*;

// Evaluation Type
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum EvalType {
    Alpha,
    Exact,
}

// Evaluation Result
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct Eval {
    pub eval_type: EvalType,
    pub value: i32,
}

impl Default for Eval {
    fn default() -> Self {
        Eval { eval_type: EvalType::Alpha,
               value: -INF_VALUE, }
    }
}

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
        if self.eval < -var.eval {
            self.eval = -var.eval;
            self.move_list = var.move_list;
            self.move_list.push_front( *mv );
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct SearchStats {
    pub end: u64,
    pub middle: u64,
    pub max_depth: u64,
    pub end_qs: u64,
    pub middle_qs: u64,
    pub quiet_qs: u64,
    pub beta_cutoff_qs: u64,
    pub hash_hit: u64,
    pub hash_cutoff: u64,
}

impl SearchStats {
    pub fn new() -> Self {
        SearchStats {
            end: 0,
            middle: 0,
            max_depth: 0,
            end_qs: 0,
            middle_qs: 0,
            quiet_qs: 0,
            beta_cutoff_qs: 0,
            hash_hit: 0,
            hash_cutoff: 0,
        }
    }
}

pub fn profile( state: &mut State, depth: usize ) {
    if depth == 0 {
        let _ = state.pst_eval();
    } else {
        let ( legal_moves, _ ) = state.node_info();
        let irs = state.ir_state();

        for mv in &legal_moves {
            state.make( mv );
            profile( state, depth - 1 );
            state.unmake( mv, &irs );
        }
    }
}

pub fn quiescence( state: &mut State, mut alpha: i32, beta: i32, stats: &mut SearchStats ) -> i32 {
    let ( legal_moves, status ) = state.node_info();

    if status == Status::Ongoing {
        let mut eval = state.pst_eval();

        if beta <= eval {
            // Assuming that we are not in Zugzwang, the "Stand Pat" is a lower bound on the eval.
            // FIXME: Ideally, we might want to make a null_move and then return that score - given we might implement some sort of Tempo into the static_eval?
            // Failing soft
            stats.beta_cutoff_qs += 1;
            eval
        } else {
            alpha = cmp::max( alpha, eval );
            let irs = state.ir_state();
            let tactical_moves = state.tactical_moves( &legal_moves );

            if tactical_moves.is_empty() { stats.quiet_qs += 1; } else { stats.middle_qs += 1; }

            for mv in &tactical_moves {
                state.make( mv );
                eval = cmp::max( eval, -quiescence( state, -beta, -alpha, stats ) );
                state.unmake( mv, &irs );

                alpha = cmp::max( alpha, eval );

                // Failing soft
                if beta <= alpha { break; }
            }

            eval
        }
    } else {
        stats.end_qs += 1;
        if status == Status::Checkmate { -MATE_VALUE } else { DRAW_VALUE }
    }
}

pub fn negamax( state: &mut State, depth: usize, mut alpha: i32, beta: i32, stats: &mut SearchStats, tt: &mut HashTable<Eval> ) -> Variation {
    let ( legal_moves, status ) = state.node_info();

    if status == Status::Ongoing {
        if let Some( hashed ) = tt.get( state.hash, depth ) {
            stats.hash_hit += 1;
            if beta <= hashed.value || ( hashed.eval_type == EvalType::Exact && hashed.value <= alpha ) {
                stats.hash_cutoff += 1;
                return Variation::terminal( hashed.value );
            }
        }

        if depth == 0 {
            stats.max_depth += 1;
            Variation::terminal( quiescence( state, alpha, beta, stats ) )
        } else {
            stats.middle += 1;
            let irs = state.ir_state();
            let mut var = Variation::terminal( -INF_VALUE );
            let mut eval_type: EvalType = EvalType::Exact;

            for mv in &legal_moves {
                state.make( mv );
                var.max_assign( mv, negamax( state, depth - 1, -beta, -alpha, stats, tt ) );
                state.unmake( mv, &irs );

                alpha = cmp::max( alpha, var.eval );

                // Failing soft
                if beta <= alpha {
                    eval_type = EvalType::Alpha;
                    break;
                }
            }

            tt.set( state.hash, depth, Eval { eval_type: eval_type, value: var.eval } );

            var
        }
    } else {
        stats.end += 1;
        if status == Status::Checkmate { Variation::terminal( -MATE_VALUE ) } else { Variation::terminal( DRAW_VALUE ) }
    }
}
