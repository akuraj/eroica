//! Simple Game Protocol

use crate::consts::*;
use crate::hashtables::*;
use crate::pgn_parser::*;
use crate::search::*;
use crate::state::*;
use std::io;

pub fn user_input<'a>(buffer: &'a mut String, stdin: &'a mut io::Stdin) -> &'a str {
    buffer.clear();
    match stdin.read_line(buffer) {
        Ok(_) => buffer.trim(),
        Err(error) => panic!("Error: {}", error),
    }
}

pub fn ask(
    buffer: &mut String,
    stdin: &mut io::Stdin,
    question: &str,
    options: &[String],
) -> String {
    println!("{}", question);
    let input = user_input(buffer, stdin).to_string();
    println!();

    if options.contains(&input) {
        input
    } else {
        ask(buffer, stdin, question, options)
    }
}

pub fn play() {
    let stdin = &mut io::stdin();
    let buffer = &mut String::new();

    if ask(
        buffer,
        stdin,
        "Wanna play a game of Chess? (y/n)",
        &["y".to_string(), "n".to_string()],
    ) == "n"
    {
        return;
    }
    let color_string = ask(
        buffer,
        stdin,
        "You wanna play White or Black? (w/b)",
        &["w".to_string(), "b".to_string()],
    );
    let opponent_color = if color_string == "w" { WHITE } else { BLACK };

    let search_depth: usize = 4;
    let mut state = State::new();
    let mut tt: HashTable<Eval> = HashTable::new(24);

    loop {
        if state.to_move == opponent_color {
            println!("{}\nYour move:\n", state);
            let input = user_input(buffer, stdin);
            if input.contains("exit") {
                break;
            }
            match parse_move(input, &state) {
                Ok(mv) => {
                    println!("Parsed: {}\n\n", mv);
                    state.make(&mv);
                }
                Err(error) => {
                    println!(
                        "\nThere was an error with the input, please try again.\nDETAIL: {}",
                        error
                    );
                    continue;
                }
            }
        } else {
            let mut stats = SearchStats::new();
            let pv = negamax(
                &mut state,
                search_depth,
                -MATE_VALUE,
                MATE_VALUE,
                &mut stats,
                &mut tt,
            );
            let mv = pv.move_list.front().unwrap();
            state.make(mv);
            println!("I just played: {}", mv);
            println!(
                "My evaluation is {}, at a depth of {}.\n",
                pv.eval, search_depth
            );
        }
    }
}
