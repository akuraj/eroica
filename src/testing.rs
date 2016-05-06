use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::str::FromStr;
use state::*;
use time::*;

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

pub fn run_perft( path: &str ) {
    // Run perft against test cases
    let file = match File::open( path ) {
        Ok( file ) => BufReader::new( file ),
        Err( error ) => panic!( "Can't find {}: {:?}", path, error ),
    };

    for line in file.lines() {
        let test = parse_peft_test_case( &line.unwrap() );
        let mut state = State::generate_state_from_fen( &test.fen );
        for item in &test.values {
            assert_eq!( state.perft( item.depth, false ), item.perft_val );
        }
    }
}

pub fn perftsuite_bench() {
    let t1 = precise_time_ns();
    run_perft( "testing/perftsuite_bench.epd" );
    let t2 = precise_time_ns();
    println!( "Time taken: {} seconds", ( ( t2 - t1 ) as f32 ) / 1e9 );
}

#[test]
pub fn perftsuite_lean() {
    run_perft( "testing/perftsuite_lean.epd" );
}

#[test]
#[ignore]
pub fn perftsuite() {
    run_perft( "testing/perftsuite.epd" );
}

#[test]
#[ignore]
pub fn perftsuite_other() {
    run_perft( "testing/perftsuite_other.epd" );
}
