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

pub mod consts;
pub mod state;
pub mod utils;
pub mod magics;

use consts::*;
use state::*;
use magics::*;
use utils::*;

fn main() {
    let mut rook_magics: [ u64; 64 ] = [ 0u64; 64 ];
    let mut bishop_magics: [ u64; 64 ] = [ 0u64; 64 ];

    for i in 0..64 {
        rook_magics[ i ] = magic( i as u32, ROOK, true );
    }

    for i in 0..64 {
        bishop_magics[ i ] = magic( i as u32, BISHOP, true );
    }

    /*
    let fen = "rn1q1rk1/p4pbp/bp1p1np1/2pP4/8/P1N2NP1/1PQ1PPBP/R1B1K2R w KQ - -";
    let state = State::generate_state_from_fen( fen );
    println!( "{}", state.bit_board[ 0 ] );
    */
}
