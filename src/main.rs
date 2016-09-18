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
pub mod simple_game;

use state::*;
use time::*;
use search::*;
use evaluation::*;
use std::io;

fn main() {
    let t1 = precise_time_ns();

    simple_game::play();

    let t2 = precise_time_ns();
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );
}
