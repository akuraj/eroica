//! State representation: Define required types and functions to construct and store the state

use std::ops::{ Index, IndexMut };
use std::default::Default;
use consts::*;

// A mailbox style board that encodes the contents of each square in u8
pub type SimpleBoard = [ u8; 64 ];

// BitBoard type, stores positions of all 12 piece types ( 6 x 2 ) + all_white and all_black as bitmaps in u64
// Have to make it a Tuple Struct because Rust doesn't allow me to implement traits I don't own ( Index, IndexMut ), on types I don't own ( [T; N] )
// Credit to "Crabby" for this hack
// FIXME: Can we do something better here?
pub struct BitBoard( pub [ u64; 14 ] );

impl Index<u8> for BitBoard {
    type Output = u64;

    fn index( &self, i: u8 ) -> &u64 {
        &self.0[ i as usize ]
    }
}

impl IndexMut<u8> for BitBoard {
    fn index_mut( &mut self, i: u8 ) -> &mut u64 {
        &mut self.0[ i as usize ]
    }
}

impl BitBoard {
    pub fn set_all( &mut self ) {
        self[ WHITE | ALL ] = self[ WHITE | PAWN ] | self[ WHITE | KNIGHT ] | self[ WHITE | BISHOP ] |
                              self[ WHITE | ROOK ] | self[ WHITE | QUEEN ] | self[ WHITE | KING ];
        self[ BLACK | ALL ] = self[ BLACK | PAWN ] | self[ BLACK | KNIGHT ] | self[ BLACK | BISHOP ] |
                              self[ BLACK | ROOK ] | self[ BLACK | QUEEN ] | self[ BLACK | KING ];
    }

    pub fn generate_bb_from_sb( sb: &SimpleBoard ) -> Self {
        let mut bb = BitBoard( [ 0u64; 14 ] );

        for ( i, &piece ) in sb.iter().enumerate() {
            if piece != EMPTY { bb[ piece ] |= 1 << i }
        }

        bb.set_all();

        bb
    }
}



pub fn algebraic_to_offset( square: &str ) -> u8 {
    if square.chars().count() != 2 {
        panic!( "Algebraic address \"{}\", does not have a size of 2!", square );
    }

    let mut char_iter = square.chars().enumerate();
    let mut offset: u8 = 0;

    while let Some( ( i, c ) ) = char_iter.next() {
        match i {
            0 => {
                if !( 'a' <= c && c <= 'h' ) {
                    panic!( "Invalid file: {}", c );
                }

                offset += c as u8 - 'a' as u8;
            },
            1 => {
                if let Some( rank_number ) = c.to_digit( 10 ) {
                    if !( 1 <= rank_number && rank_number <= 8 ) {
                        panic!( "Invalid file: {}", c );
                    }

                    offset += ( rank_number as u8 - 1 ) * 8;
                }
                else
                {
                    panic!( "Invalid rank: {}", c );
                }
            },
            _ => panic!( "Invalid algebraic address \"{}\"", square ),
        }
    }

    offset
}

// Full state
pub struct State {
    pub simple_board: SimpleBoard,
    pub bit_board: BitBoard,
    pub to_move: u8,
    pub castling: u8,
    pub en_passant: u64, // FIXME: Maybe imlement a logic which can handle a u8?
    pub halfmove_clock: u8,

    // repetition table

    // hash
    // other bb items
}

impl Default for State {
    fn default() -> Self {
        State { simple_board: [ EMPTY; 64 ],
                bit_board: BitBoard( [ 0u64; 14 ] ),
                to_move: 0,
                castling: 0,
                en_passant: 0,
                halfmove_clock: 0, }
    }
}

impl State {
    pub fn generate_state_from_fen( fen: &str ) -> Self {
        // Check that fen specifies the required number of fields
        // NOTE: We DON'T check for legality of the position
        match fen.split_whitespace().count() {
            5 | 6 => {},
            _ => panic!( "\nFEN must specify exactly 5 or 6 fields ( if fullmove_count is also included ):\n{}", fen ),
        }

        let mut iter = fen.split_whitespace().enumerate();
        let mut state = State{ ..Default::default() };

        while let Some( ( section_number, section ) ) = iter.next() {
            match section_number {
                0 => {
                    // position

                    // Populate simple_board
                    if section.rsplit( '/' ).count() != 8 {
                        panic!( "\nPosition should contain 8 rows:\n{}", section );
                    }

                    let mut row_iter = section.rsplit( '/' ).enumerate();
                    let mut position_offset: usize = 0;

                    while let Some( ( row_number, row ) ) = row_iter.next() {
                        if position_offset != 8 * row_number {
                            panic!( "\nInvalid position_offset: {}, at row_number: {}", position_offset, row_number );
                        }

                        let mut char_iter = row.chars();

                        while let Some( c ) = char_iter.next() {
                            if let Some( empty_space ) = c.to_digit( 10 ) {
                                position_offset += empty_space as usize;
                            }
                            else
                            {
                                state.simple_board[ position_offset ] = match c {
                                    'P' => WHITE | PAWN,
                                    'N' => WHITE | KNIGHT,
                                    'B' => WHITE | BISHOP,
                                    'R' => WHITE | ROOK,
                                    'Q' => WHITE | QUEEN,
                                    'K' => WHITE | KING,
                                    'p' => BLACK | PAWN,
                                    'n' => BLACK | KNIGHT,
                                    'b' => BLACK | BISHOP,
                                    'r' => BLACK | ROOK,
                                    'q' => BLACK | QUEEN,
                                    'k' => BLACK | KING,
                                     _  => panic!( "Invalid piece: {}", c ),
                                };

                                position_offset += 1;
                            }
                        }
                    }

                    // Populate bit_board
                    state.bit_board = BitBoard::generate_bb_from_sb( &state.simple_board );
                },
                1 => {
                    // to_move
                    state.to_move = match section {
                        "w" => WHITE,
                        "b" => BLACK,
                         _  => panic!( "to_move is invalid: {}", section ),
                    }
                },
                2 => {
                    // castling
                    match section {
                        "-" => {},
                         _  => {
                             let mut char_iter = section.chars();
                             while let Some( c ) = char_iter.next() {
                                 state.castling |= match c {
                                     'K' => WK_CASTLE,
                                     'Q' => WQ_CASTLE,
                                     'k' => BK_CASTLE,
                                     'q' => BQ_CASTLE,
                                      _  => panic!( "Invalid castling token: {}", c ),
                                 }
                             }
                         }
                    }
                },
                3 => {
                    // en_passant
                    state.en_passant = match section {
                        "-" => 0,
                         _  => 1 << algebraic_to_offset( section ),
                    }
                },
                4 => {
                    // halfmove_clock
                    state.halfmove_clock = match section {
                        "-" => 0,
                         _  => section.parse::<u8>().unwrap(),
                    }
                },
                _ => {},
            }
        }

        state
    }
}
