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
    assert!( square.chars().count() == 2, "Invalid algebraic address \"{}\"", square );
    let mut char_iter = square.chars().enumerate();
    let mut offset: u8 = 0;

    while let Some( ( i, c ) ) = char_iter.next() {
        match i {
            0 => {
                assert!( 'a' <= c && c <= 'h', "Invalid file: {}", c );
                offset += c as u8 - 'a' as u8;
            },
            1 => {
                if let Some( rank_number ) = c.to_digit( 10 ) {
                    assert!( 1 <= rank_number && rank_number <= 8, "Invalid rank: {}", c );
                    offset += ( rank_number as u8 - 1 ) * 8;
                } else {
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
    pub en_passant: u8, // Store the file index
    pub halfmove_clock: u8,
    pub fullmove_count: u8,

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
                en_passant: EMPTY,
                halfmove_clock: 0,
                fullmove_count: 0, }
    }
}

impl State {
    pub fn generate_state_from_fen( fen: &str ) -> Self {
        // Check that fen specifies the required number of fields
        let num_fields = fen.split_whitespace().count();
        assert!( num_fields == 5 || num_fields == 6, "FEN must specify exactly 5 or 6 fields ( if fullmove_count is also included ):\n{}", fen );

        let mut iter = fen.split_whitespace().enumerate();
        let mut state = State{ ..Default::default() };

        while let Some( ( section_number, section ) ) = iter.next() {
            match section_number {
                0 => {
                    // position

                    // Populate simple_board
                    assert!( section.rsplit( '/' ).count() == 8, "Position should contain 8 rows:\n{}", section );
                    let mut row_iter = section.rsplit( '/' ).enumerate();
                    let mut position_offset: usize = 0;

                    while let Some( ( row_number, row ) ) = row_iter.next() {
                        assert!( position_offset == 8 * row_number, "Invalid position_offset: {}, at row_number: {}", position_offset, row_number );
                        let mut char_iter = row.chars();

                        while let Some( c ) = char_iter.next() {
                            if let Some( empty_space ) = c.to_digit( 10 ) {
                                position_offset += empty_space as usize;
                            } else {
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
                        "-" => EMPTY,
                         _  => algebraic_to_offset( section ) % 8,
                    }
                },
                4 => {
                    // halfmove_clock
                    state.halfmove_clock = match section {
                        "-" => 0,
                         _  => section.parse::<u8>().unwrap(),
                    }
                },
                5 => {
                    // fullmove_count
                    state.fullmove_count = match section {
                        "-" => 0,
                         _  => section.parse::<u8>().unwrap(),
                    }
                },
                _ => {},
            }
        }

        // FIXME: Implement a state check

        state
    }
}

// A simple fn to print a BitBoard
pub fn print_bb( bb: &u64 ) {
    let top = " ___ ___ ___ ___ ___ ___ ___ ___ \n";
    let start  = "|   |   |   |   |   |   |   |   |\n";
    let end  = "\n|___|___|___|___|___|___|___|___|\n";

    let mut output = String::new();
    let mut row: u64 = FIRST_RANK << 56;
    let mut read: u64;

    output.push_str( top );

    for j in 0..8 {
        output.push_str( start );
        output.push( '|' );
        read = ( bb & row ) >> ( 56 - 8 * j );

        for i in 0..8 {
            if read & ( 1 << i ) != 0 {
                output.push_str( " x |" );
            } else {
                output.push_str( "   |" );
            }
        }

        output.push_str( end );
        row >>= 8;
    }

    println!( "{}", output );
}

// ( file, rank )
pub fn file_rank( pos: u32 ) -> ( u32, u32 ) {
    ( pos % 8, pos / 8 )
}

// Rook and Bishop Masks
pub fn rook_mask( pos: u32 ) -> u64 {
    let ( i, j ) = file_rank( pos );
    ( A_FILE_NE << i ) ^ ( FIRST_RANK_NE << ( 8 * j ) )
}

pub fn bishop_mask( pos: u32 ) -> u64 {
    let ( i, j ) = file_rank( pos );
    let s = i + j;
    let mut bishop_mask: u64;

    if i > j {
        bishop_mask = ( A1_H8 << ( i - j ) ) & LRT;
    }
    else {
        bishop_mask = ( A1_H8 >> ( j - i ) ) & ULT;
    }

    if s > 7 {
        bishop_mask ^= A8_H1 << ( 8 * s - 56 );
    }
    else {
        bishop_mask ^= A8_H1 >> ( 56 - 8 * s );
    }

    bishop_mask & NOT_EDGES
}
