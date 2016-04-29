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
pub mod attacks;

use consts::*;
use state::*;
use magics::*;
use utils::*;
use time::*;
use attacks::*;

fn main() {
    let mut att = Attacks { ..Default::default() };
    att.init( true );

    let fen = "rn1q1rk1/p4pbp/bp1p1np1/2pP4/8/P1N2NP1/1PQ1PPBP/R1B1K2R w KQ - -";
    //let fen = "8/6bb/8/8/R1pP2k1/4P3/P7/K7 b - d3 -";
    let mut state = State::generate_state_from_fen( fen );
    println!( "{}", state );

    let occ = state.bit_board[ WHITE_ALL ] | state.bit_board[ BLACK_ALL ];

    let t1 = precise_time_ns();
    let mut rmoves: u64;
    //rmoves = att.b_moves( 37, occ );

    for _ in 0..100000000 {
        for pos in 0..64 {
            rmoves = att.b_moves( pos, occ );
        }
    }

    //print_bb( &rmoves );

    let t2 = precise_time_ns();
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );

    /*
    //let fen = "rn1q1rk1/p4pbp/bp1p1np1/2pP4/8/P1N2NP1/1PQ1PPBP/R1B1K2R w KQ - -";
    let fen = "8/6bb/8/8/R1pP2k1/4P3/P7/K7 b - d3 -";
    let mut state = State::generate_state_from_fen( fen );
    let mv = Move { piece: BLACK_KING, from: 30, to: 21, capture: EMPTY };
    let irs = state.ir_state();

    for i in 0..1000000000 {
        state.make( &mv );
        state.unmake( &mv, &irs );
    }
    */

    /*
    let mv = Move { piece: WHITE_KING, from: 4, to: 6, capture: EMPTY };
    let irs = state.ir_state();
    state.make( &mv );
    println!( "{}", state );
    state.unmake( &mv, &irs );
    println!( "{}", state );
    */
}
