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
    let state = State::generate_state_from_fen( fen );
    //println!( "{:?}", state );
    let t2 = precise_time_ns();
    println!( "\n\nTime taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );

    /*
    let t1 = precise_time_ns();
    //check_stored_magics( ROOK );
    //check_stored_magics( BISHOP );
    let mut x: u64;
    for i in 0..64 {
        x = magic( i as u32, ROOK, false );
        x = magic( i as u32, BISHOP, false );
    }
    let t2 = precise_time_ns();
    println!( "\n\nTime taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );
    */

    /*
    let fen = "rn1q1rk1/p4pbp/bp1p1np1/2pP4/8/P1N2NP1/1PQ1PPBP/R1B1K2R w KQ - -";
    let state = State::generate_state_from_fen( fen );
    println!( "{}", state.bit_board[ 0 ] );
    */
}
