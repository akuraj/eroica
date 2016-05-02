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
pub mod movegen;

use consts::*;
use state::*;
use magics::*;
use utils::*;
use time::*;
use movegen::*;
use rand::{ Rng, thread_rng };

fn main() {
    let fen = START_FEN; // until 7
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - -"; // Kiwipete, until 5
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - -"; // until 7
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // until 6
    // let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"; // until 6 - MIRROR OF THE ABOVE
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"; // until 5, apparently this was hard? meh..
    // let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"; // checks out until 6 - very slow...
    // let fen = "8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - - -"; // until 8
    // let fen = "r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - - -"; // until 6
    // let fen = "8/5p2/8/2k3P1/p3K3/8/1P6/8 b - - - -"; // until 8
    // let fen = "r3k2r/pb3p2/5npp/n2p4/1p1PPB2/6P1/P2N1PBP/R3K2R b KQkq - - -"; // until 6

    let mut state = State::generate_state_from_fen( fen );

    /*
    let mv = Move { piece: WHITE_ROOK, from: algebraic_to_offset( "b4" ), to: algebraic_to_offset( "a4" ), capture: EMPTY, promotion: EMPTY };
    state.make( &mv );

    let mv = Move { piece: BLACK_ROOK, from: algebraic_to_offset( "h5" ), to: algebraic_to_offset( "c5" ), capture: EMPTY, promotion: EMPTY };
    state.make( &mv );

    let mv = Move { piece: WHITE_KING, from: algebraic_to_offset( "a5" ), to: algebraic_to_offset( "b4" ), capture: EMPTY, promotion: EMPTY };
    state.make( &mv );

    let mv = Move { piece: BLACK_ROOK, from: algebraic_to_offset( "c5" ), to: algebraic_to_offset( "c1" ), capture: EMPTY, promotion: EMPTY };
    state.make( &mv );

    let mv = Move { piece: WHITE_ROOK, from: algebraic_to_offset( "a4" ), to: algebraic_to_offset( "a6" ), capture: EMPTY, promotion: EMPTY };
    state.make( &mv );

    let mv = Move { piece: BLACK_PAWN, from: algebraic_to_offset( "c7" ), to: algebraic_to_offset( "c5" ), capture: EMPTY, promotion: EMPTY };
    state.make( &mv );
    */

    println!( "{}", state.fen() );
    println!( "{}", state );

    let depth: usize = 6;
    let t1 = precise_time_ns();
    let perft_val = state.perft( depth, true );
    let t2 = precise_time_ns();
    println!( "Perft {}: {}", depth, perft_val );
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );

    //println!( "Speed: {} MNPS", ( ( loop_size as f32 * moves.len() as f32 ) / ( ( ( t2 - t1 ) as f32 ) / 1e9 ) ) / 1e6 );

    //let fen = "rn1q1rk1/p4pbp/bp1p1np1/2pP4/8/P1N2NP1/1PQ1PPBP/R1B1K2R w KQ - -";
    //let fen = "8/6bb/8/8/R2P2k1/4P3/P1p5/K2B4 b - d3 -";
}
