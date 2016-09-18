//! Simple Game Protocol

use std::io;
use state::*;
use evaluation::*;
use search::*;
use consts::*;
use pgn_parser::*;

pub fn user_input< 'a >( buffer: &'a mut String, stdin: &'a mut io::Stdin ) -> &'a str {
    buffer.clear();
    match stdin.read_line( buffer ) {
        Ok( _ ) => buffer.trim(),
        Err( error ) => panic!( "Error: {}", error ),
    }
}

pub fn ask( buffer: &mut String, stdin: &mut io::Stdin, question: &str, options: &[ String ] ) -> String {
    println!( "{}", question );
    let input = user_input( buffer, stdin ).to_string();
    println!( "" );

    if options.contains( &input ) {
        input
    } else {
        ask( buffer, stdin, question, options )
    }
}

pub fn play() {
    let stdin = &mut io::stdin();
    let buffer = &mut String::new();

    if ask( buffer, stdin, "Wanna play a game of Chess? (y/n)", &[ "y".to_string(), "n".to_string() ] ) == "n" { return (); }
    let color_string = ask( buffer, stdin, "You wanna play White or Black? (w/b)", &[ "w".to_string(), "b".to_string() ] );
    let opponent_color = if color_string == "w" { WHITE } else { BLACK };

    let search_depth: usize = 4;
    let mut state = State::new();

    loop {
        if state.to_move == opponent_color {
            println!( "{}\nYour move:\n", state );
            let input = user_input( buffer, stdin );
            match parse_move( input, &state ) {
                Ok( mv ) => {
                    println!( "Parsed: {}\n\n", mv );
                    state.make( &mv );
                },
                Err( error ) => {
                    println!( "\nThere was an error with the input, please try again.\nDETAIL: {}", error );
                    continue;
                },
            }
        } else {
            let pv = negamax( &mut state, search_depth, -MATE_VALUE, MATE_VALUE );
            let mv = pv.move_list.get( 0 ).unwrap();
            state.make( mv );
            println!( "I just played: {}", mv );
            println!( "My evaluation is {}, at a depth of {}.\n", pv.eval, search_depth );
        }
    }
}
