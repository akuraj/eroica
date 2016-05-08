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

use consts::*;
use state::*;
use magics::*;
use utils::*;
use time::*;
use movegen::*;
use testing::*;
use hash::*;

fn main() {
    perftsuite_bench();
}
