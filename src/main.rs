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
use hashtables::*;

fn main() {
    let t1 = precise_time_ns();

    let mut state = State::new();
    let mut stats = SearchStats::new();
    let mut tt = TranspositionTable::new( 24 );
    let pv = negamax( &mut state, 9, -INF_VALUE, INF_VALUE, &mut stats, &mut tt );
    println!( "Eval: {}\n", pv.eval );
    println!( "{:?}\n", stats );

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

    // simple_game::play();

    let t2 = precise_time_ns();
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );
}
