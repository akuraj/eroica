#![feature(specialization)]
#[cfg(test)]

extern crate rand;
extern crate time;
extern crate rand_chacha;

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

use std::time::Instant;
use state::*;
use consts::*;
use search::*;
use hashtables::*;

fn main() {
    //testing::perftsuite_bench();
    // NOTE: Test Commit!

    let start = Instant::now();

    let mut state = State::new();
    let mut stats = SearchStats::new();
    let mut tt: HashTable<Eval> = HashTable::new( 24 );
    //println!( "{}\n", state );
    let pv = negamax( &mut state, 8, -INF_VALUE, INF_VALUE, &mut stats, &mut tt );
    println!( "Eval: {}\n", pv.eval );
    //println!( "{:?}\n", stats );
    //println!( "{:?}\n", pv.move_list );

    println!( "Time taken: {} seconds", ( start.elapsed().as_nanos() as f32 ) / 1e9 );

    //simple_game::play();

    //let fen = "1rbq1rk1/p1b1nppp/1p2p3/8/1B1pN3/P2B4/1P3PPP/2RQ1R1K w - - 0 0";
    //let mut state = State::generate_state_from_fen( fen );

    //let mut state = State::new();
    //profile( &mut state, 6 );

    /*
    let fen = "r1bqr1k1/pp1n1ppp/3bp3/8/3pB3/2P2N2/PP3PPP/R1BQR1K1 w - - 0 12";
    let mv_str = "Bxh7+";
    let mut state = State::generate_state_from_fen( fen );
    let mv = pgn_parser::parse_move( mv_str, &state ).unwrap();
    let irs = state.ir_state();

    for _ in 0..100000000 {
        state.make( &mv );
        state.unmake( &mv, &irs );
    }
    */
}
