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
#![feature(test)]
extern crate rand;
extern crate time;
extern crate test;

pub mod consts;
pub mod state;
pub mod utils;
pub mod magics;
pub mod movegen;
pub mod testing;
pub mod hash;
pub mod hashtables;
pub mod pgn_parser;
pub mod evaluation;

use consts::*;
use state::*;
use utils::*;
use time::*;
use testing::*;
use pgn_parser::*;

fn main() {
    perftsuite_bench();
    let _ = parse_pgn( "testing/r1000.pgn" );
}
