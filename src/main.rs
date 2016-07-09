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
