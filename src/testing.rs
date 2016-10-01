use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::str::FromStr;
use state::*;
use time::*;
use pgn_parser::*;

#[derive(Debug)]
pub struct PerftResultItem {
    pub depth: usize,
    pub perft_val: u64,
}

#[derive(Debug)]
pub struct PerftResult {
    pub fen: String,
    pub values: Vec<PerftResultItem>,
}

pub fn parse_peft_test_case( test: &str ) -> PerftResult {
    let mut iter = test.split( " ;" );

    if let Some( fen ) = iter.next() {
        let mut values: Vec<PerftResultItem> = Vec::new();

        for item in iter {
            let mut iter_item = item.split_whitespace();
            let depth_opt = iter_item.next();
            let perft_val_opt = iter_item.next();

            match ( depth_opt, perft_val_opt ) {
                ( Some( depth ), Some( perft_val ) ) => {
                    let mut depth_str = String::from( depth );
                    assert_eq!( depth_str.remove( 0 ), 'D' );
                    values.push( PerftResultItem { depth: usize::from_str( &depth_str ).unwrap(), perft_val: u64::from_str( perft_val ).unwrap() } );
                },
                _ => panic!( "Invalid test case: {}", item ),
            }
        }

        assert!( values.len() > 0, "No perft info to test for: {}", fen );
        PerftResult { fen: String::from( fen ), values: values }
    } else {
        panic!( "Test case is empty!" );
    }
}

pub fn run_perft( path: &str, use_hash: bool ) {
    // Run perft against test cases
    let file = match File::open( path ) {
        Ok( file ) => BufReader::new( file ),
        Err( error ) => panic!( "Can't find {}: {:?}", path, error ),
    };

    for line in file.lines() {
        let test = parse_peft_test_case( &line.unwrap() );
        let mut state = State::generate_state_from_fen( &test.fen );
        for item in &test.values {
            if use_hash {
                assert_eq!( state.hash_perft( item.depth, false ), item.perft_val );
            } else {
                assert_eq!( state.perft( item.depth, false ), item.perft_val );
            }
        }
    }
}

pub fn run_check_hash_rec( path: &str ) {
    // Run check_hash_rec against test cases
    let file = match File::open( path ) {
        Ok( file ) => BufReader::new( file ),
        Err( error ) => panic!( "Can't find {}: {:?}", path, error ),
    };

    for line in file.lines() {
        let test = parse_peft_test_case( &line.unwrap() );
        let mut state = State::generate_state_from_fen( &test.fen );
        let max_depth = test.values.iter().fold( 0, |acc, x| if x.depth > acc { x.depth } else { acc } );
        assert!( state.check_hash_rec( max_depth ) );
    }
}

pub fn run_check_pst_eval_rec( path: &str ) {
    // Run check_hash_rec against test cases
    let file = match File::open( path ) {
        Ok( file ) => BufReader::new( file ),
        Err( error ) => panic!( "Can't find {}: {:?}", path, error ),
    };

    for line in file.lines() {
        let test = parse_peft_test_case( &line.unwrap() );
        let mut state = State::generate_state_from_fen( &test.fen );
        let max_depth = test.values.iter().fold( 0, |acc, x| if x.depth > acc { x.depth } else { acc } );
        assert!( state.check_pst_eval_rec( max_depth ) );
    }
}

pub fn perftsuite_bench() {
    let t1 = precise_time_ns();
    run_perft( "testing/perftsuite_bench.epd", true );
    let t2 = precise_time_ns();
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );
}

pub fn game_termination_rep() {
    let _ = parse_pgn( "testing/r1000.pgn" );
}

#[test]
pub fn perftsuite_lean() {
    run_perft( "testing/perftsuite_lean.epd", true );
}

#[test]
pub fn game_termination() {
    game_termination_rep()
}

#[test]
#[ignore]
pub fn perftsuite_no_hash() {
    run_perft( "testing/perftsuite.epd", false );
}

#[test]
#[ignore]
pub fn perftsuite() {
    run_perft( "testing/perftsuite.epd", true );
}

#[test]
#[ignore]
pub fn perftsuite_other() {
    run_perft( "testing/perftsuite_other.epd", true );
}

#[test]
#[ignore]
pub fn test_check_hash_rec() {
    run_check_hash_rec( "testing/perftsuite_lean.epd" );
}

#[test]
#[ignore]
pub fn test_check_pst_eval_rec() {
    run_check_pst_eval_rec( "testing/perftsuite_lean.epd" );
}
