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
pub mod search;

use state::*;
use time::*;
use search::*;
use evaluation::*;

fn main() {
    let mut state = State::new();
    let t1 = precise_time_ns();
    let eval = negamax( &mut state, 8, -MATE_VALUE, MATE_VALUE );
    let t2 = precise_time_ns();
    println!( "Eval: {}", eval );
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );
}
