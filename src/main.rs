/*

 function negamax(node, depth, α, β, color)
     if depth = 0 or node is a terminal node
         return color * the heuristic value of node // symmetric eval fn

     childNodes := GenerateMoves(node)
     childNodes := OrderMoves(childNodes)
     bestValue := −∞
     foreach child in childNodes
         v := −negamax(child, depth − 1, −β, −α, −color)
         bestValue := max( bestValue, v )
         α := max( α, v )
         if α ≥ β // can also fail hard if you want
             break
     return bestValue

     // TT should store alpha, beta, bestValue???
     // also return the principal variation

*/

/*
cargo rustc --release -- -C target-feature=+popcnt -C target-cpu=native
*/

extern crate rand;
extern crate time;

pub mod consts;
pub mod state;
pub mod utils;
pub mod magics;

use consts::*;
use state::*;
use magics::*;
use utils::*;
use time::*;

fn main() {
    let t1 = precise_time_ns();

    let fen = "rn1q1rk1/p4pbp/bp1p1np1/2pP4/8/P1N2NP1/1PQ1PPBP/R1B1K2R w KQ - -";
    let mut state = State::generate_state_from_fen( fen );
    println!( "{}", state );

    let t2 = precise_time_ns();
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );

    /*
    let mv = Move { piece: WHITE_KING, from: 4, to: 6, capture: EMPTY };
    let irs = state.ir_state();
    state.make( &mv );
    println!( "{}", state );
    state.unmake( &mv, &irs );
    println!( "{}", state );
    */
}
