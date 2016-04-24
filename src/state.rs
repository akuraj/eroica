//! State representation: Define required types and functions to construct and store the state

use std::ops::{ Index, IndexMut };
use std::default::Default;
use consts::*;
use utils::*;

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

// Move
pub struct Move {
    pub piece: u8,
    pub from: u8,
    pub to: u8,
    pub capture: u8,
}

impl Move {
    pub fn castling( &self ) -> u8 {
        // Assumes that the moving piece is a King
        match self.from {
            4 => match self.to {
                2 => WQ_CASTLE,
                6 => WK_CASTLE,
                _ => 0,
            },
            60 => match self.to {
                58 => BQ_CASTLE,
                62 => BK_CASTLE,
                _ => 0,
            },
            _ => 0,
        }
    }
}

// Irreversible State
pub struct IRState {
    pub castling: u8,
    pub en_passant: u8, // Store the pos (square address)
    pub halfmove_clock: u8,
}

// Full state
pub struct State {
    pub simple_board: SimpleBoard,
    pub bit_board: BitBoard,
    pub to_move: u8,
    pub castling: u8,
    pub en_passant: u8, // Store the pos (square address)
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
                         _  => algebraic_to_offset( section ),
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

    pub fn make( &mut self, mv: &Move ) {
        let side = self.to_move;
        let opp_side = side ^ 1u8;

        assert!( ( mv.piece & side ) != 0, "Why are you moving the opponent's pieces?" );

        // Update simple_board
        self.simple_board[ mv.from as usize ] = EMPTY;
        self.simple_board[ mv.to as usize ] = mv.piece;

        // Update bit_board
        let mv_bb: u64 = ( 1u64 << mv.from ) | ( 1u64 << mv.to );
        self.bit_board[ mv.piece ] ^= mv_bb;
        if mv.capture != EMPTY {
            let mv_bb_opp: u64 = 1u64 << mv.to;
            self.bit_board[ mv.capture ] ^= mv_bb_opp;
        }

        // Update castling state and en_passant
        // Update simple_board and bit_board for Rook if castling
        let mut new_ep = EMPTY;
        match side {
            WHITE => {
                if ( mv.piece & PAWN ) != 0 {
                    match mv.to - mv.from {
                        8 => {}, // forward_1, nothing to do
                        16 => { new_ep = mv.from + 8; }, // forward_2, set en_passant capture
                        _ => {
                            if self.en_passant == mv.to {
                                // en_passant capture
                                self.simple_board[ ( mv.to - 8 ) as usize ] = EMPTY;
                                self.bit_board[ BLACK | PAWN ] ^= 1u64 << ( mv.to - 8 );
                            }
                        },
                    }
                } else if ( mv.piece & KING ) != 0 {
                    match mv.castling() {
                        WK_CASTLE => {
                            self.simple_board[ 7 ] = EMPTY;
                            self.simple_board[ 5 ] = WHITE | ROOK;
                            self.bit_board[ WHITE | ROOK ] ^= ROOK_WKC;
                        },
                        WQ_CASTLE => {
                            self.simple_board[ 0 ] = EMPTY;
                            self.simple_board[ 3 ] = WHITE | ROOK;
                            self.bit_board[ WHITE | ROOK ] ^= ROOK_WQC;
                        },
                        _ => {},
                    };

                    self.castling &= !W_CASTLE;
                } else if ( mv.piece & ROOK ) != 0 {
                    match mv.from {
                        0 => self.castling &= !WQ_CASTLE,
                        7 => self.castling &= !WK_CASTLE,
                        _ => {},
                    }
                }
            },
            BLACK => {
                if ( mv.piece & PAWN ) != 0 {
                    match mv.from - mv.to {
                        8 => {}, // forward_1, nothing to do
                        16 => { new_ep = mv.to + 8; }, // forward_2, set en_passant capture
                        _ => {
                            if self.en_passant == mv.to {
                                // en_passant capture
                                self.simple_board[ ( mv.to + 8 ) as usize ] = EMPTY;
                                self.bit_board[ WHITE | PAWN ] ^= 1u64 << ( mv.to + 8 );
                            }
                        },
                    }
                } else if ( mv.piece & KING ) != 0 {
                    match mv.castling() {
                        BK_CASTLE => {
                            self.simple_board[ 63 ] = EMPTY;
                            self.simple_board[ 61 ] = BLACK | ROOK;
                            self.bit_board[ BLACK | ROOK ] ^= ROOK_BKC;
                        },
                        BQ_CASTLE => {
                            self.simple_board[ 56 ] = EMPTY;
                            self.simple_board[ 59 ] = BLACK | ROOK;
                            self.bit_board[ BLACK | ROOK ] ^= ROOK_BQC;
                        },
                        _ => {},
                    };

                    self.castling &= !B_CASTLE;
                } else if ( mv.piece & ROOK ) != 0 {
                    match mv.from {
                        56 => self.castling &= !BQ_CASTLE,
                        63 => self.castling &= !BK_CASTLE,
                        _ => {},
                    }
                }
            },
            _ => panic!( "Invalid color!" ),
        }

        self.bit_board.set_all(); // update 'ALL' bit_boards
        self.en_passant = new_ep; // set en_passant
        self.to_move = opp_side; // set side
        if side == BLACK { self.fullmove_count += 1; } // update fullmove_count
        if mv.piece == ( side | PAWN ) || mv.capture != EMPTY { self.halfmove_clock = 0; } else { self.halfmove_clock += 1; } // update halfmove_clock

        // update repetition table
        // update other blah
    }

    pub fn unmake( &mut self, mv: &Move, irs: &IRState ) {

    }
}
