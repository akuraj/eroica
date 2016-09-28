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
pub mod simple_game;

use time::*;
use state::*;
use consts::*;
use search::*;

fn main() {
    let t1 = precise_time_ns();

    let mut state = State::new();
    let pv = negamax( &mut state, 7, -MATE_VALUE, MATE_VALUE );
    println!( "Eval: {}", pv.eval );

    // simple_game::play();

    let t2 = precise_time_ns();
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );
}
