//! Implement a pgn_parser

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use crate::state::*;
use crate::consts::*;
use crate::utils::*;

// Game Start Delineator
pub const PGN_DELINEATOR: &'static str = "[Event";

// Game Result Strings
pub const WHITE_WON: &'static str = "\"1-0\"";
pub const BLACK_WON: &'static str = "\"0-1\"";
pub const GAME_DRAWN: &'static str = "\"1/2-1/2\"";
pub const GAME_ONGOING: &'static str = "\"*\"";

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum GameResult {
    WhiteWon,
    BlackWon,
    Drawn,
    Ongoing, // Ongoing, Unknown or Abandoned
}

#[derive(Clone,Debug,PartialEq)]
pub struct Game {
    pub init_pos: String, // Start Position as FEN
    pub move_list: Vec<Move>,
    pub result: GameResult, // Recorded result
    pub end_status: Status, // Objective Status
}

pub fn oc( mt: &mut String, oc: &mut bool ) {
    assert!( !( *oc ) );
    *oc = true;
    mt.push_str( " \" " );
}

pub fn parse_move( mv_str: &str, state: &State ) -> Result< Move, String > {
    let ( legal_moves, _ ) = state.node_info();

    match mv_str {
        "O-O" | "O-O-O" => {
            let mut mv = match state.to_move {
                WHITE => {
                    let mut mv_c = Move::null_move( WHITE_KING, 4 );
                    mv_c.to = if mv_str == "O-O" { 6 } else { 2 };
                    mv_c
                },
                BLACK => {
                    let mut mv_c = Move::null_move( BLACK_KING, 60 );
                    mv_c.to = if mv_str == "O-O" { 62 } else { 58 };
                    mv_c
                },
                _ => panic!( "Invalid side to move!" ),
            };

            state.evaluate_move( &mut mv );
            if legal_moves.contains( &mv ) {
                Ok( mv )
            } else {
                Err( format!( "Illegal move: {}", mv ) )
            }
        },
        _ => {
            let mut mv_str_mut: String = mv_str.to_string();

            // Check
            let check_str: String = mv_str_mut.chars().filter( |&x| x == '+' ).collect();
            let check_count = check_str.chars().count();
            if check_count > 1 { return Err( format!( "Too many '+'s in {}", mv_str_mut ) ); }
            let is_check = check_count > 0;
            if is_check && !mv_str_mut.ends_with( &check_str ) {
                return Err( format!( "{} doesn't end with {}", mv_str_mut, check_str ) );
            }

            // Checkmate
            let checkmate_str: String = mv_str_mut.chars().filter( |&x| x == '#' ).collect();
            let checkmate_count = checkmate_str.chars().count();
            if checkmate_count > 1 { return Err( format!( "Too many '#'s in {}", mv_str_mut ) ); }
            let is_checkmate = checkmate_count > 0;
            if is_checkmate && !mv_str_mut.ends_with( &checkmate_str ) {
                return Err( format!( "{} doesn't end with {}", mv_str_mut, checkmate_str ) );
            }

            if is_check && is_checkmate { return Err( format!( "Input move has both '+' and '#'" ) ); }

            // Remove check and checkmate
            mv_str_mut = mv_str_mut.chars().filter( |&x| x != '+' && x != '#' ).collect();

            let piece_char = match mv_str_mut.chars().nth( 0 ) {
                Some( piece_char_actual ) => piece_char_actual,
                None => return Err( format!( "piece_char: mv_str_mut[ 0 ] out of bounds!" ) ),
            };

            let from: usize;
            let to: usize;
            let mut capture = EMPTY;
            let mut promotion = EMPTY;

            let piece = state.to_move | match piece_char {
                'K' => KING,
                'Q' => QUEEN,
                'R' => ROOK,
                'B' => BISHOP,
                'N' => KNIGHT,
                _ => {
                    if 'a' <= piece_char && piece_char <= 'h' {
                        PAWN
                    } else {
                        return Err( format!( "Invalid piece_char: {}", piece_char ) )
                    }
                },
            };

            // Handle promotion
            if piece == ( state.to_move | PAWN ) {
                let promotion_str: String = mv_str_mut.chars().filter( |&x| x == '=' ).collect();
                let promotion_count = promotion_str.chars().count();
                if promotion_count > 1 { return Err( format!( "Too many '='s in {}", mv_str_mut ) ); }
                let is_promotion = promotion_count > 0;
                if is_promotion {
                    let temp_str = mv_str_mut;

                    let promoted_to = match temp_str.split( '=' ).nth( 1 ) {
                        Some( promoted_to_actual ) => promoted_to_actual,
                        None => return Err( format!( "promoted_to: temp_str[ 1 ] out of bounds!" ) ),
                    };

                    mv_str_mut = ( match temp_str.split( '=' ).nth( 0 ) {
                        Some( mv_str_mut_actual ) =>  mv_str_mut_actual,
                        None => return Err( format!( "temp_str[ 0 ] out of bounds!" ) ),
                    } ).to_string();

                    promotion = state.to_move | match promoted_to {
                        "Q" => QUEEN,
                        "R" => ROOK,
                        "B" => BISHOP,
                        "N" => KNIGHT,
                        _ => return Err( format!( "Invalid promotion: {}", promoted_to ) ),
                    };
                }
            }

            // Handle capture and to location
            let capture_str: String = mv_str_mut.chars().filter( |&x| x == 'x' ).collect();
            let capture_count = capture_str.chars().count();
            if capture_count > 1 { return Err( format!( "Too many 'x's in the input move: {}", mv_str_mut ) ); }
            let is_capture = capture_count > 0;
            if is_capture {
                let temp_str = mv_str_mut;

                let capture_square = match temp_str.split( 'x' ).nth( 1 ) {
                    Some( capture_square_actual ) => capture_square_actual,
                    None => return Err( format!( "capture_square: temp_str[ 1 ] out of bounds!" ) ),
                };

                mv_str_mut = ( match temp_str.split( 'x' ).nth( 0 ) {
                    Some( mv_str_mut_actual ) =>  mv_str_mut_actual,
                    None => return Err( format!( "temp_str[ 0 ] out of bounds!" ) ),
                } ).to_string();

                to = algebraic_to_offset( capture_square );
                capture = state.simple_board[ to ];
            } else {
                let temp_str = mv_str_mut;
                let size = temp_str.chars().count();
                let destination: String = temp_str.chars().skip( size - 2 ).collect();
                to = algebraic_to_offset( &destination );
                mv_str_mut = temp_str.chars().take( size - 2 ).collect();
            }

            // Handle disambiguation
            let mut possibilities: Vec<Move> = Vec::new();
            for x in legal_moves.iter() {
                if x.piece == piece && x.to == to && x.capture == capture && x.promotion == promotion {
                    possibilities.push( *x );
                }
            }

            let num_poss = possibilities.iter().count();

            if num_poss > 1 {
                let mut poss_filtered: Vec<Move> = Vec::new();

                if piece == ( state.to_move | PAWN ) {
                    let size = mv_str_mut.chars().count();
                    if size != 1 {
                        return Err( format!( "Disambiguation is problematic 1: {}, {}", mv_str, mv_str_mut ) );
                    } else {
                        let file = match mv_str_mut.chars().nth( 0 ) {
                            Some( file_actual ) => file_actual,
                            None => return Err( format!( "file: mv_str_mut[ 0 ] out of bounds!" ) ),
                        };

                        if 'a' <= file && file <= 'h' {
                            let file_num = file as usize - 'a' as usize;
                            for x in possibilities.iter() {
                                if x.from % 8 == file_num {
                                    poss_filtered.push( *x );
                                }
                            }
                        } else {
                            return Err( format!( "Invalid file: {}", file ) );
                        }
                    }
                } else {
                    mv_str_mut = mv_str_mut.chars().skip( 1 ).collect(); // Remove the piece identifier
                    let size = mv_str_mut.chars().count();
                    if size > 2 {
                        return Err( format!( "Disambiguation is problematic 2: {}, {}", mv_str, mv_str_mut ) );
                    } else if size == 2 {
                        from = algebraic_to_offset( &mv_str_mut );
                        for x in possibilities.iter() {
                            if x.from == from {
                                poss_filtered.push( *x );
                            }
                        }
                    } else if size == 1 {
                        let disamb = match mv_str_mut.chars().nth( 0 ) {
                            Some( file_actual ) => file_actual,
                            None => return Err( format!( "disamb: mv_str_mut[ 0 ] out of bounds!" ) ),
                        };

                        if '1' <= disamb && disamb <= '8' {
                            let rank_num = disamb as usize - '1' as usize;
                            for x in possibilities.iter() {
                                if x.from / 8 == rank_num {
                                    poss_filtered.push( *x );
                                }
                            }
                        } else if 'a' <= disamb && disamb <= 'h' {
                            let file_num = disamb as usize - 'a' as usize;
                            for x in possibilities.iter() {
                                if x.from % 8 == file_num {
                                    poss_filtered.push( *x );
                                }
                            }
                        } else {
                            return Err( format!( "Disambiguation is problematic 3: {}, {}", mv_str, mv_str_mut ) )
                        }
                    } else {
                        return Err( format!( "Disambiguation is problematic 4: {}, {}", mv_str, mv_str_mut ) )
                    }
                }

                let final_size = poss_filtered.iter().count();
                if final_size == 1 {
                    match poss_filtered.iter_mut().nth( 0 ) {
                        Some( poss_filtered_actual ) => {
                            state.evaluate_move( poss_filtered_actual );
                            Ok( *poss_filtered_actual )
                        },
                        None => return Err( format!( "poss_filtered: poss_filtered[ 0 ] out of bounds!" ) ),
                    }
                } else {
                    return Err( format!( "Disambiguation is problematic 5: {}, {}", mv_str, mv_str_mut ) )
                }
            } else if num_poss == 1 {
                match possibilities.iter_mut().nth( 0 ) {
                    Some( possibilities_actual ) => {
                        state.evaluate_move( possibilities_actual );
                        Ok( *possibilities_actual )
                    },
                    None => return Err( format!( "possibilities: possibilities[ 0 ] out of bounds!" ) ),
                }
            } else {
                return Err( format!( "Illegal move: {}", mv_str ) )
            }
        }
    }
}

// Parse pgn, do some checks and return the GameList
pub fn parse_pgn( path: &str ) -> Vec<Game> {
    let file = match File::open( path ) {
        Ok( file ) => BufReader::new( file ),
        Err( error ) => panic!( "Can't find {}: {:?}", path, error ),
    };

    // get file as String
    let mut file_string = String::new();
    for line in file.lines() {
        file_string.push_str( "\n" );
        file_string.push_str( &line.unwrap() );
    }

    let mut games: Vec<Game> = Vec::new();

    // Get a game iterator
    let mut game_iter = file_string.split( PGN_DELINEATOR ).skip( 1 );
    while let Some( pgn ) = game_iter.next() {
        let pgn = pgn.trim();
        let move_text = pgn.split( "]" ).last().unwrap();
        let mut curr_iter = pgn.split( "]" ).skip( 1 );

        // Seven Tag Roster - we already ignored 'Event'
        // NOTE: We are going to ignore the contents of most of the tags
        let site = curr_iter.next().unwrap().trim();
        assert!( site.starts_with( "[Site" ) );

        let date = curr_iter.next().unwrap().trim();
        assert!( date.starts_with( "[Date" ) );

        let round = curr_iter.next().unwrap().trim();
        assert!( round.starts_with( "[Round" ) );

        let white = curr_iter.next().unwrap().trim();
        assert!( white.starts_with( "[White" ) );

        let black = curr_iter.next().unwrap().trim();
        assert!( black.starts_with( "[Black" ) );

        let result = curr_iter.next().unwrap().trim();
        assert!( result.starts_with( "[Result" ) );

        let result_val = result.split_whitespace().last().unwrap();
        let result_val_nq = result_val.split( "\"" ).nth( 1 ).unwrap();
        assert_eq!( move_text.split_whitespace().last().unwrap(), result_val_nq );
        let result_enum = match result_val {
            WHITE_WON => GameResult::WhiteWon,
            BLACK_WON => GameResult::BlackWon,
            GAME_DRAWN => GameResult::Drawn,
            GAME_ONGOING => GameResult::Ongoing,
            _ => panic!( "Invalid game result: {}", result_val ),
        };

        let mut fen: String = START_FEN.to_string();
        let mut set_up: bool = false;
        let mut termination: String = "".to_string();
        let mut final_fen: String = "".to_string();

        while let Some( item ) = curr_iter.next() {
            let item = item.trim();
            if item.starts_with( "[SetUp" ) {
                if item.split( "\"" ).nth( 1 ).unwrap() == "1" {
                    set_up = true;
                }
            } else if item.starts_with( "[FEN" ) {
                fen = item.split( "\"" ).nth( 1 ).unwrap().to_string();
            } else if item.starts_with( "[FinalFEN" ) {
                final_fen = item.split( "\"" ).nth( 1 ).unwrap().to_string();
            } else if item.starts_with( "[Termination" ) {
                termination = item.split( "\"" ).nth( 1 ).unwrap().to_string();
            }
        }

        if !set_up && fen != START_FEN {
            panic!( "If not Set Up - fen has to be START_FEN!" );
        }

        // Replace any comment/annotation/variation with the comment indicator, '"'
        // We just ignore all the above mentioned stuff
        // Nested stuff is unsupported!
        // Too lazy to write a proper parser....
        let all_special: String = "\"{}()%".to_string();
        let move_text_nr = move_text.split( result_val_nq ).nth( 0 ).unwrap().trim();
        let mut move_text_pure = String::new();
        let mut open_comment: bool = false;
        let mut comment_type: char = '"';
        for elem in move_text_nr.chars() {
            if open_comment {
                if elem == comment_type {
                    open_comment = false;
                } else if all_special.contains( elem ) {
                    panic!( "We don't support nested variations/comments!" );
                }
            } else {
                match elem {
                    '"' => {
                        oc( &mut move_text_pure, &mut open_comment );
                        comment_type = '"';
                    },
                    '{' => {
                        oc( &mut move_text_pure, &mut open_comment );
                        comment_type = '}';
                    },
                    '(' => {
                        oc( &mut move_text_pure, &mut open_comment );
                        comment_type = ')';
                    },
                    '%' => {
                        oc( &mut move_text_pure, &mut open_comment );
                        comment_type = '\n';
                    },
                    _ => {
                        move_text_pure.push( elem );
                    },
                }
            }
        }

        let mut state = State::generate_state_from_fen( &fen );
        let mut move_list: Vec<Move> = Vec::new();

        // Parse move_text_pure
        let mut mni: bool = true; // move number indication
        for mv_str in move_text_pure.split_whitespace() {
            if mv_str == "\"" {
                mni = true;
            } else if mni {
                let number: String = mv_str.chars().filter( |&x| x != '.' ).collect();
                let dots: String = mv_str.chars().filter( |&x| x == '.' ).collect();
                assert_eq!( number.parse::<usize>().unwrap(), state.fullmove_count );
                assert!( mv_str.ends_with( &dots ) );
                match state.to_move {
                    WHITE => assert_eq!( dots, "." ),
                    BLACK => assert_eq!( dots, "..." ),
                    _ => panic!( "Invalid side!" ),
                }

                mni = false;
            } else { // This one is a move... finally!
                let the_move = match parse_move( mv_str, &state ) {
                    Ok( the_move_actual ) => the_move_actual,
                    Err( error ) => panic!( "{}", error ),
                };

                move_list.push( the_move );
                state.make( &the_move );
                mni = state.to_move == WHITE;

                if mv_str.ends_with( '+' ) {
                    assert!( state.num_checks > 0 );
                } else if mv_str.ends_with( '#' ) {
                    assert!( state.num_checks > 0 );
                    let ( legal_moves, _ ) = state.node_info();
                    assert_eq!( legal_moves.iter().count(), 0 );
                }
            }
        }

        if final_fen != "" {
            assert_eq!( final_fen, state.fen( true ) );
        }

        let ( _, status ) = state.node_info();

        if termination.contains( "checkmate" ) {
            assert_eq!( status, Status::Checkmate );
        } else if termination.contains( "stalemate" ) {
            assert_eq!( status, Status::Stalemate );
        } else if termination.contains( "fifty move rule" ) {
            assert_eq!( status, Status::FiftyMoveDraw );
        } else if termination.contains( "repetition" ) {
            assert_eq!( status, Status::RepetitionDraw );
        } else if termination.contains( "insufficient material" ) {
            assert_eq!( status, Status::InsufficientMaterial );
        }

        games.push( Game { init_pos: fen,
                           move_list: move_list,
                           result: result_enum,
                           end_status: status } );
    }

    games
}
