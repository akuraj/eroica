//! State representation: Define required types and functions to construct and store the state

use std::ops::{ Index, IndexMut };
use std::default::Default;
use std::fmt;
use consts::*;
use utils::*;
use movegen::*;
use hash::*;
use hashtables::*;

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
        self[ WHITE_ALL ] = self[ WHITE_PAWN ] | self[ WHITE_KNIGHT ] | self[ WHITE_BISHOP ] |
                              self[ WHITE_ROOK ] | self[ WHITE_QUEEN ] | self[ WHITE_KING ];
        self[ BLACK_ALL ] = self[ BLACK_PAWN ] | self[ BLACK_KNIGHT ] | self[ BLACK_BISHOP ] |
                              self[ BLACK_ROOK ] | self[ BLACK_QUEEN ] | self[ BLACK_KING ];
    }

    pub fn generate_bb_from_sb( sb: &SimpleBoard ) -> Self {
        let mut bb = BitBoard( [ 0; 14 ] );

        for ( i, &piece ) in sb.iter().enumerate() {
            if piece != EMPTY { bb[ piece ] |= 1 << i }
        }

        bb.set_all();

        bb
    }
}

// Move
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Move {
    pub piece: u8,
    pub from: usize,
    pub to: usize,
    pub capture: u8,
    pub promotion: u8,
}

impl Move {
    pub fn castling( &self ) -> u8 {
        match ( self.piece, self.from ) {
            ( WHITE_KING, 4 ) => match self.to {
                2 => WQ_CASTLE,
                6 => WK_CASTLE,
                _ => 0,
            },
            ( BLACK_KING, 60 ) => match self.to {
                58 => BQ_CASTLE,
                62 => BK_CASTLE,
                _ => 0,
            },
            _ => 0,
        }
    }

    pub fn castling_path( &self ) -> u64 {
        match ( self.piece, self.from ) {
            ( WHITE_KING, 4 ) => match self.to {
                2 => WQCR,
                6 => WKCR,
                _ => 0,
            },
            ( BLACK_KING, 60 ) => match self.to {
                58 => BQCR,
                62 => BKCR,
                _ => 0,
            },
            _ => 0,
        }
    }

    pub fn is_promotion( &self ) -> bool {
        match ( self.piece, self.from / 8 ) {
            ( WHITE_PAWN, 6 ) | ( BLACK_PAWN, 1 ) => true,
            _ => false,
        }
    }

    // A bb of what changed - ignoring en_passant
    pub fn move_bb( &self ) -> u64 {
        ( 1 << self.from ) | ( 1 << self.to )
    }
}

// Game status enum
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Status {
    Unknown,
    Ongoing,
    Checkmate,
    Stalemate
}

// NodeInfo
#[derive(Clone,Debug)]
pub struct NodeInfo {
    pub status: Status,
    pub legal_moves: Vec<Move>,
}

impl Default for NodeInfo {
    fn default() -> Self {
        NodeInfo { status: Status::Unknown,
                   legal_moves: Vec::new() }
    }
}

// Irreversible State - and also Control info, which is expensive to recalc
pub struct IRState {
    // IRState
    pub castling: u8,
    pub en_passant: usize, // Store the pos (square address)
    pub halfmove_clock: u8,

    // Control
    pub attacked: u64,
    pub num_checks: u8,
    pub check_blocker: u64,
    pub defended: u64,
    pub a_pins: [ u64; 64 ],
    pub control: [ u64; 64 ],
    pub ep_possible: bool,

    // Hash
    pub hash: u64,
}

// Full State
pub struct State {
    // State (the board) + IRState
    pub simple_board: SimpleBoard,
    pub bit_board: BitBoard,
    pub to_move: u8,
    pub castling: u8,
    pub en_passant: usize, // Store the pos (square address)
    pub halfmove_clock: u8,
    pub fullmove_count: u8, // Starts at 1 (First Move)

    // Move Generator
    pub mg: MoveGen,

    // Control
    pub attacked: u64,
    pub num_checks: u8,
    pub check_blocker: u64,
    pub defended: u64,
    pub a_pins: [ u64; 64 ],
    pub control: [ u64; 64 ],
    pub ep_possible: bool,

    // Hash Generator
    pub hg: HashGen,

    // Hash
    pub hash: u64,

    // repetition table
    // other bb items
}

impl Default for State {
    fn default() -> Self {
        State { simple_board: [ EMPTY; 64 ],
                bit_board: BitBoard( [ 0; 14 ] ),
                to_move: 0,
                castling: 0,
                en_passant: ERR_POS,
                halfmove_clock: 0,
                fullmove_count: 0,
                mg: MoveGen { ..Default::default() },
                attacked: 0,
                num_checks: 0,
                check_blocker: FULL_BOARD,
                defended: 0,
                a_pins: [ FULL_BOARD; 64 ],
                control: [ 0; 64 ],
                ep_possible: false,
                hg: HashGen { ..Default::default() },
                hash: 0, }
    }
}

impl fmt::Display for State {
    fn fmt( &self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();

        // Board
        let top = " ___ ___ ___ ___ ___ ___ ___ ___ \n";
        let start  = "|   |   |   |   |   |   |   |   |\n";
        let end  = "\n|___|___|___|___|___|___|___|___|\n";

        output.push_str( top );

        for j in ( 0..8 ).rev() {
            output.push_str( start );
            output.push( '|' );

            for i in 0..8 {
                output.push( ' ' );
                output.push( piece_to_char( self.simple_board[ i + 8 * j ] ) );
                output.push_str( " |" );
            }

            output.push_str( end );
        }

        output.push( '\n' );

        // Rest of the State
        match self.to_move {
            WHITE => output.push_str( "To move: White\n" ),
            BLACK => output.push_str( "To move: Black\n" ),
            _ => panic!( "Invalid color!" ),
        };

        output.push_str( "Castling: " );
        if self.castling & WK_CASTLE != 0 { output.push( 'K') }
        if self.castling & WQ_CASTLE != 0 { output.push( 'Q') }
        if self.castling & BK_CASTLE != 0 { output.push( 'k') }
        if self.castling & BQ_CASTLE != 0 { output.push( 'q') }
        output.push( '\n' );

        output.push_str( "En Passant: " );
        if self.ep_flag() { output.push_str( &offset_to_algebraic( self.en_passant ) ) }
        output.push( '\n' );

        output.push_str( "Halfmove Clock: " );
        output.push_str( &( self.halfmove_clock.to_string() ) );
        output.push( '\n' );

        output.push_str( "Fullmove Count: " );
        output.push_str( &( self.fullmove_count.to_string() ) );
        output.push( '\n' );

        write!( f, "{}", output )
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
                                state.simple_board[ position_offset ] = char_to_piece( c );
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
                        "-" => ERR_POS,
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
                        "-" => 1,
                         _  => section.parse::<u8>().unwrap(),
                    }
                },
                _ => {},
            }
        }

        state.compute_control();
        state.state_check();
        state.set_hash();

        state
    }

    pub fn state_check( &self ) {
        // Very basic checks -> does not assert that the position is a child of START_FEN (can be reached from the initial position)
        // Sufficient for the engine to play

        // Check number of kings
        assert_eq!( self.bit_board[ WHITE | KING ].count_ones(), 1 );
        assert_eq!( self.bit_board[ BLACK | KING ].count_ones(), 1 );

        // Check that opposing king is not in check if it's our move
        assert_eq!( self.defended & self.bit_board[ ( self.to_move ^ COLOR ) | KING ], 0 );

        // castling
        if self.castling & W_CASTLE != 0 {
            assert_eq!( self.simple_board[ 4 ], WHITE_KING );

            if self.castling & WK_CASTLE != 0 {
                assert_eq!( self.simple_board[ 7 ], WHITE_ROOK );
            }

            if self.castling & WQ_CASTLE != 0 {
                assert_eq!( self.simple_board[ 0 ], WHITE_ROOK );
            }
        }

        if self.castling & B_CASTLE != 0 {
            assert_eq!( self.simple_board[ 60 ], BLACK_KING );

            if self.castling & BK_CASTLE != 0 {
                assert_eq!( self.simple_board[ 63 ], BLACK_ROOK );
            }

            if self.castling & BQ_CASTLE != 0 {
                assert_eq!( self.simple_board[ 56 ], BLACK_ROOK );
            }
        }

        // en_passant
        if self.en_passant != ERR_POS {
            match self.to_move {
                WHITE => {
                    assert_eq!( self.en_passant / 8, 5 );
                    assert_eq!( self.simple_board[ self.en_passant - 8 ], BLACK_PAWN );
                    assert_eq!( self.simple_board[ self.en_passant ], EMPTY );
                    assert_eq!( self.simple_board[ self.en_passant + 8 ], EMPTY );
                },
                BLACK => {
                    assert_eq!( self.en_passant / 8, 2 );
                    assert_eq!( self.simple_board[ self.en_passant + 8 ], WHITE_PAWN );
                    assert_eq!( self.simple_board[ self.en_passant ], EMPTY );
                    assert_eq!( self.simple_board[ self.en_passant - 8 ], EMPTY );
                },
                _ => {},
            }
        }
    }

    pub fn fen( &self ) -> String {
        let mut output = String::new();

        output.push( '"' );

        // Board
        let mut counter: usize;
        let mut piece: u8;
        for j in ( 0..8 ).rev() {
            counter = 0;
            for i in 0..8 {
                piece = self.simple_board[ i + 8 * j ];
                if piece == EMPTY {
                    counter += 1;
                } else if counter > 0 {
                    output.push_str( &counter.to_string() );
                    output.push( piece_to_char( piece ) );
                    counter = 0;
                } else {
                    output.push( piece_to_char( piece ) );
                }
            }

            if counter > 0 {
                output.push_str( &counter.to_string() );
            }
            if j != 0 {
                output.push( '/' );
            }
        }

        // Side to move
        match self.to_move {
            WHITE => output.push_str( " w " ),
            BLACK => output.push_str( " b " ),
            _ => panic!( "Invalid color!" ),
        }

        // Castling
        if self.castling & WK_CASTLE != 0 { output.push( 'K') }
        if self.castling & WQ_CASTLE != 0 { output.push( 'Q') }
        if self.castling & BK_CASTLE != 0 { output.push( 'k') }
        if self.castling & BQ_CASTLE != 0 { output.push( 'q') }
        output.push( ' ' );

        // ep
        if self.ep_flag() {
            output.push_str( &offset_to_algebraic( self.en_passant ) );
        } else {
            output.push( '-' );
        }
        output.push( ' ' );

        // halfmove_clock
        output.push_str( &self.halfmove_clock.to_string() );
        output.push( ' ' );

        // fullmove_count
        output.push_str( &self.fullmove_count.to_string() );

        output.push( '"' );
        output
    }

    pub fn new_game() -> Self {
        State::generate_state_from_fen( START_FEN )
    }

    pub fn set_ir_state( &mut self, irs: &IRState ) {
        self.castling = irs.castling;
        self.en_passant = irs.en_passant;
        self.halfmove_clock = irs.halfmove_clock;
        self.attacked = irs.attacked;
        self.num_checks = irs.num_checks;
        self.check_blocker = irs.check_blocker;
        self.defended = irs.defended;
        self.a_pins = irs.a_pins;
        self.control = irs.control;
        self.ep_possible = irs.ep_possible;
        self.hash = irs.hash;
    }

    pub fn ir_state( &self ) -> IRState {
        IRState{ castling: self.castling,
                 en_passant: self.en_passant,
                 halfmove_clock: self.halfmove_clock,
                 attacked: self.attacked,
                 num_checks: self.num_checks,
                 check_blocker: self.check_blocker,
                 defended: self.defended,
                 a_pins: self.a_pins,
                 control: self.control,
                 ep_possible: self.ep_possible,
                 hash: self.hash, }
    }

    pub fn make( &mut self, mv: &Move ) {
        // We only make legal moves!
        let side = self.to_move;
        self.hash ^= self.hg.side_hash; // HASH_UPDATE

        // Remove old castling and ep from hash
        self.hash ^= self.hg.castling( self.castling ); // HASH_UPDATE
        if self.ep_possible {
            self.hash ^= self.hg.ep( self.en_passant ); // HASH_UPDATE
        }

        // Update simple_board and bit_board
        self.simple_board[ mv.from ] = EMPTY;
        self.simple_board[ mv.to ] = mv.piece;
        self.bit_board[ mv.piece ] ^= mv.move_bb();
        self.hash ^= self.hg.piece( mv.piece, mv.from ); // HASH_UPDATE
        self.hash ^= self.hg.piece( mv.piece, mv.to ); // HASH_UPDATE
        if mv.capture != EMPTY {
            self.bit_board[ mv.capture ] ^= 1 << mv.to;
            self.hash ^= self.hg.piece( mv.capture, mv.to ); // HASH_UPDATE
        }

        // Update castling state and en_passant; handle promotion
        // Update simple_board and bit_board for Rook if castling
        let mut new_ep = ERR_POS;
        match mv.piece {
            WHITE_PAWN => {
                if mv.is_promotion() {
                    self.simple_board[ mv.to ] = mv.promotion;
                    self.bit_board[ WHITE_PAWN ] ^= 1 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1 << mv.to;
                    self.hash ^= self.hg.piece( WHITE_PAWN, mv.to ); // HASH_UPDATE
                    self.hash ^= self.hg.piece( mv.promotion, mv.to ); // HASH_UPDATE
                } else {
                    match mv.to - mv.from {
                        16 => { new_ep = mv.from + 8; }, // forward_2, set en_passant capture
                        _ => {
                            if self.en_passant == mv.to {
                                let ep_target = mv.to - 8;
                                self.simple_board[ ep_target ] = EMPTY;
                                self.bit_board[ BLACK_PAWN ] ^= 1 << ep_target;
                                self.hash ^= self.hg.piece( BLACK_PAWN, ep_target ); // HASH_UPDATE
                            }
                        },
                    }
                }
            },
            WHITE_KING => {
                match mv.castling() {
                    WK_CASTLE => {
                        self.simple_board[ 7 ] = EMPTY;
                        self.simple_board[ 5 ] = WHITE_ROOK;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WKC;
                        self.hash ^= self.hg.piece( WHITE_ROOK, 7 ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( WHITE_ROOK, 5 ); // HASH_UPDATE
                    },
                    WQ_CASTLE => {
                        self.simple_board[ 0 ] = EMPTY;
                        self.simple_board[ 3 ] = WHITE_ROOK;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WQC;
                        self.hash ^= self.hg.piece( WHITE_ROOK, 0 ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( WHITE_ROOK, 3 ); // HASH_UPDATE
                    },
                    _ => {},
                };

                self.castling &= !W_CASTLE;
            },
            WHITE_ROOK => {
                match mv.from {
                    0 => self.castling &= !WQ_CASTLE,
                    7 => self.castling &= !WK_CASTLE,
                    _ => {},
                }
            },
            BLACK_PAWN => {
                if mv.is_promotion() {
                    self.simple_board[ mv.to ] = mv.promotion;
                    self.bit_board[ BLACK_PAWN ] ^= 1 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1 << mv.to;
                    self.hash ^= self.hg.piece( BLACK_PAWN, mv.to ); // HASH_UPDATE
                    self.hash ^= self.hg.piece( mv.promotion, mv.to ); // HASH_UPDATE
                } else {
                    match mv.from - mv.to {
                        16 => { new_ep = mv.to + 8; }, // forward_2, set en_passant capture
                        _ => {
                            if self.en_passant == mv.to {
                                let ep_target = mv.to + 8;
                                self.simple_board[ ep_target ] = EMPTY;
                                self.bit_board[ WHITE_PAWN ] ^= 1 << ep_target;
                                self.hash ^= self.hg.piece( WHITE_PAWN, ep_target ); // HASH_UPDATE
                            }
                        },
                    }
                }
            },
            BLACK_KING => {
                match mv.castling() {
                    BK_CASTLE => {
                        self.simple_board[ 63 ] = EMPTY;
                        self.simple_board[ 61 ] = BLACK_ROOK;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BKC;
                        self.hash ^= self.hg.piece( BLACK_ROOK, 63 ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( BLACK_ROOK, 61 ); // HASH_UPDATE
                    },
                    BQ_CASTLE => {
                        self.simple_board[ 56 ] = EMPTY;
                        self.simple_board[ 59 ] = BLACK_ROOK;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BQC;
                        self.hash ^= self.hg.piece( BLACK_ROOK, 56 ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( BLACK_ROOK, 59 ); // HASH_UPDATE
                    },
                    _ => {},
                };

                self.castling &= !B_CASTLE;
            },
            BLACK_ROOK => {
                match mv.from {
                    56 => self.castling &= !BQ_CASTLE,
                    63 => self.castling &= !BK_CASTLE,
                    _ => {},
                }
            },
            _ => {},
        }

        // Rook Capture (update Castling)
        match mv.capture {
            WHITE_ROOK => {
                match mv.to {
                    0 => self.castling &= !WQ_CASTLE,
                    7 => self.castling &= !WK_CASTLE,
                    _ => {},
                }
            },
            BLACK_ROOK => {
                match mv.to {
                    56 => self.castling &= !BQ_CASTLE,
                    63 => self.castling &= !BK_CASTLE,
                    _ => {},
                }
            },
            _ => {},
        }

        self.bit_board.set_all(); // update 'ALL' bit_boards
        self.en_passant = new_ep; // set en_passant
        self.to_move ^= COLOR; // set side
        if side == BLACK { self.fullmove_count += 1; } // update fullmove_count
        if mv.piece == ( side | PAWN ) || mv.capture != EMPTY { self.halfmove_clock = 0; } else { self.halfmove_clock += 1; } // update halfmove_clock

        self.compute_control();

        // Add new castling and ep to hash
        self.hash ^= self.hg.castling( self.castling ); // HASH_UPDATE
        if self.ep_possible {
            self.hash ^= self.hg.ep( self.en_passant ); // HASH_UPDATE
        }

        // update repetition table
        // update other blah
    }

    pub fn unmake( &mut self, mv: &Move, irs: &IRState ) {
        let side = self.to_move ^ COLOR; // side that just moved

        // Update simple_board and bit_board
        self.simple_board[ mv.from ] = mv.piece;
        self.simple_board[ mv.to ] = mv.capture;
        self.bit_board[ mv.piece ] ^= mv.move_bb();
        if mv.capture != EMPTY { self.bit_board[ mv.capture ] ^= 1 << mv.to; }

        // Undo castling and en_passant
        match mv.piece {
            WHITE_PAWN => {
                if mv.is_promotion() {
                    self.bit_board[ WHITE_PAWN ] ^= 1 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1 << mv.to;
                } else if irs.en_passant == mv.to {
                    self.simple_board[ mv.to - 8 ] = BLACK_PAWN;
                    self.bit_board[ BLACK_PAWN ] ^= 1 << ( mv.to - 8 );
                }
            },
            WHITE_KING => {
                match mv.castling() {
                    WK_CASTLE => {
                        self.simple_board[ 7 ] = WHITE_ROOK;
                        self.simple_board[ 5 ] = EMPTY;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WKC;
                    },
                    WQ_CASTLE => {
                        self.simple_board[ 0 ] = WHITE_ROOK;
                        self.simple_board[ 3 ] = EMPTY;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WQC;
                    },
                    _ => {},
                }
            },
            BLACK_PAWN => {
                if mv.is_promotion() {
                    self.bit_board[ BLACK_PAWN ] ^= 1 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1 << mv.to;
                } else if irs.en_passant == mv.to {
                    self.simple_board[ mv.to + 8 ] = WHITE_PAWN;
                    self.bit_board[ WHITE_PAWN ] ^= 1 << ( mv.to + 8 );
                }
            },
            BLACK_KING => {
                match mv.castling() {
                    BK_CASTLE => {
                        self.simple_board[ 63 ] = BLACK_ROOK;
                        self.simple_board[ 61 ] = EMPTY;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BKC;
                    },
                    BQ_CASTLE => {
                        self.simple_board[ 56 ] = BLACK_ROOK;
                        self.simple_board[ 59 ] = EMPTY;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BQC;
                    },
                    _ => {},
                }
            },
            _ => {},
        }

        self.set_ir_state( irs );

        self.bit_board.set_all(); // update 'ALL' bit_boards
        self.to_move ^= COLOR; // set side
        if side == BLACK { self.fullmove_count -= 1; } // update fullmove_count

        // update repetition table
        // update other blah
    }

    pub fn ep_flag( &self ) -> bool {
        self.en_passant != ERR_POS
    }

    pub fn ep_bb( &self ) -> u64 {
        if self.ep_flag() {
            1 << self.en_passant
        } else {
            0
        }
    }

    pub fn ep_target( &self ) -> usize {
        match ( self.to_move, self.ep_flag() ) {
            ( WHITE, true ) => self.en_passant - 8,
            ( BLACK, true ) => self.en_passant + 8,
            _ => ERR_POS,
        }
    }

    pub fn ep_target_bb( &self ) -> u64 {
        if self.ep_flag() {
            1 << self.ep_target()
        } else {
            0
        }
    }

    pub fn null_move( &self, piece: u8, pos: usize ) -> Move {
        Move { piece: piece,
               from: pos,
               to: ERR_POS,
               capture: EMPTY,
               promotion: EMPTY, }
    }

    // All pseudo-legal moves for a given piece at a given pos, added to moves
    pub fn add_moves_from_bb( &self, piece: u8, pos: usize, moves_bb: &mut u64, moves: &mut Vec<Move> ) {
        let mut mv = self.null_move( piece, pos );
        let mut to: usize;

        if mv.is_promotion() {
            let side = piece & COLOR;
            while *moves_bb != 0 {
                to = pop_lsb_pos( moves_bb );
                mv.to = to;
                mv.capture = self.simple_board[ to ];

                mv.promotion = side | QUEEN;
                moves.push( mv );
                mv.promotion = side | ROOK;
                moves.push( mv );
                mv.promotion = side | BISHOP;
                moves.push( mv );
                mv.promotion = side | KNIGHT;
                moves.push( mv );
            }
        } else {
            while *moves_bb != 0 {
                to = pop_lsb_pos( moves_bb );
                mv.to = to;
                mv.capture = self.simple_board[ to ];

                moves.push( mv );
            }
        }
    }

    // All pseudo-legal moves
    pub fn moves( &self ) -> Vec<Move> {
        let side = self.to_move;
        let opp_side = side ^ COLOR;

        let not_friendly = !self.bit_board[ side | ALL ];
        let occupancy = self.bit_board[ side | ALL ] | self.bit_board[ opp_side | ALL ];
        let occupancy_w_ep = occupancy | self.ep_bb();

        let mut piece: u8;
        let mut bb: u64;
        let mut moves_bb: u64;
        let mut pos: usize;
        let mut moves: Vec<Move> = Vec::new();

        // QUEEN
        piece = side | QUEEN;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.q_moves( pos, occupancy ) & not_friendly;
            self.add_moves_from_bb( piece, pos, &mut moves_bb, &mut moves );
        }

        // ROOK
        piece = side | ROOK;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.r_moves( pos, occupancy ) & not_friendly;
            self.add_moves_from_bb( piece, pos, &mut moves_bb, &mut moves );
        }

        // BISHOP
        piece = side | BISHOP;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.b_moves( pos, occupancy ) & not_friendly;
            self.add_moves_from_bb( piece, pos, &mut moves_bb, &mut moves );
        }

        // KNIGHT
        piece = side | KNIGHT;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.n_moves( pos ) & not_friendly;
            self.add_moves_from_bb( piece, pos, &mut moves_bb, &mut moves );
        }

        // PAWN
        piece = side | PAWN;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.p_moves( pos, side, occupancy_w_ep ) & not_friendly;
            self.add_moves_from_bb( piece, pos, &mut moves_bb, &mut moves );
        }

        // KING
        piece = side | KING;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.k_moves( pos, side, occupancy, self.castling ) & not_friendly;
            self.add_moves_from_bb( piece, pos, &mut moves_bb, &mut moves );
        }

        moves
    }

    pub fn compute_control( &mut self ) {
        self.attacked = 0;
        self.num_checks = 0;
        self.check_blocker = FULL_BOARD;
        self.defended = 0;
        self.a_pins = [ FULL_BOARD; 64 ];
        self.control = [ 0; 64 ];
        self.ep_possible = false;

        let side = self.to_move;
        let opp_side = side ^ COLOR;
        let king = self.bit_board[ side | KING ];
        let king_pos = king.trailing_zeros() as usize;
        let opp_king = self.bit_board[ opp_side | KING ];
        let opp_king_pos = opp_king.trailing_zeros() as usize;
        let friends = self.bit_board[ side | ALL ];
        let enemies = self.bit_board[ opp_side | ALL ];
        let occupancy = friends | enemies;

        // So, the king, if in check is going to be blocking some sliding piece attacks, and we need to account for that.
        // This wouldn't come up when computing control of friendlies, because, if it's my move, then the enemy king can't be in check..
        let occupancy_wo_king = occupancy ^ king;
        let occupancy_wo_opp_king = occupancy ^ opp_king;

        let mut bb: u64;
        let mut pos: usize;
        let mut o_attacks: u64;
        let mut r_attacks: u64;
        let mut b_attacks: u64;

        /**** Enemies ****/

        // ROOK & QUEEN - orthogonal attacks
        bb = self.bit_board[ opp_side | ROOK ] | self.bit_board[ opp_side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            r_attacks = self.mg.r_moves( pos, occupancy_wo_king );
            self.control[ pos ] |= r_attacks;
            self.attacked |= r_attacks;

            if r_attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= line_segment( pos, king_pos ) ^ king;
            }
        }

        // BISHOP & QUEEN - diagonal attacks
        bb = self.bit_board[ opp_side | BISHOP ] | self.bit_board[ opp_side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            b_attacks = self.mg.b_moves( pos, occupancy_wo_king );
            self.control[ pos ] |= b_attacks;
            self.attacked |= b_attacks;

            if b_attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= line_segment( pos, king_pos ) ^ king;
            }
        }

        // KNIGHT
        bb = self.bit_board[ opp_side | KNIGHT ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.n_moves( pos );
            self.control[ pos ] |= o_attacks;
            self.attacked |= o_attacks;

            if o_attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= 1 << pos;
            }
        }

        // PAWN
        bb = self.bit_board[ opp_side | PAWN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.p_captures( pos, opp_side );
            self.control[ pos ] |= o_attacks;
            self.attacked |= o_attacks;

            if o_attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= 1 << pos;
            }
        }

        // KING
        bb = self.bit_board[ opp_side | KING ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.k_captures( pos );
            self.control[ pos ] |= o_attacks;
            self.attacked |= o_attacks;
        }

        /**** Friends ****/

        // ROOK & QUEEN - orthogonal attacks
        bb = self.bit_board[ side | ROOK ] | self.bit_board[ side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            r_attacks = self.mg.r_moves( pos, occupancy );
            self.control[ pos ] |= r_attacks;
            self.defended |= r_attacks;
        }

        // BISHOP & QUEEN - diagonal attacks
        bb = self.bit_board[ side | BISHOP ] | self.bit_board[ side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            b_attacks = self.mg.b_moves( pos, occupancy );
            self.control[ pos ] |= b_attacks;
            self.defended |= b_attacks;
        }

        // KNIGHT
        bb = self.bit_board[ side | KNIGHT ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.n_moves( pos );
            self.control[ pos ] |= o_attacks;
            self.defended |= o_attacks;
        }

        // PAWN
        bb = self.bit_board[ side | PAWN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.p_captures( pos, side );
            self.control[ pos ] |= o_attacks;
            self.defended |= o_attacks;
        }

        // KING
        bb = self.bit_board[ side | KING ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.k_captures( pos );
            self.control[ pos ] |= o_attacks;
            self.defended |= o_attacks;
        }

        assert!( self.defended & opp_king == 0, "Enemy King is in check - it could be Checkmate - or the last move was illegal!" );

        /**** Pins ****/

        let mut vision: u64;
        let mut possibly_pinned: u64;
        let mut possibly_attacking: u64;
        let mut pinners: u64;
        let mut pinned_pos: usize;
        let mut pin: u64;

        /* Diagonal pins */

        // Our King
        vision = self.mg.b_moves( king_pos, occupancy_wo_king );
        possibly_pinned = vision & friends;
        if possibly_pinned != 0 {
            possibly_attacking = vision & enemies;
            pinners = self.mg.b_moves( king_pos, occupancy_wo_king ^ possibly_pinned ) &
                      ( ( self.bit_board[ opp_side | QUEEN ] | self.bit_board[ opp_side | BISHOP ] ) & !possibly_attacking );
            while pinners != 0 {
                pos = pop_lsb_pos( &mut pinners );
                pin = line_segment( pos, king_pos ) ^ king;
                pinned_pos = ( possibly_pinned & pin ).trailing_zeros() as usize;
                self.a_pins[ pinned_pos ] &= pin;
            }
        }

        // Enemy King
        vision = self.mg.b_moves( opp_king_pos, occupancy_wo_opp_king );
        possibly_pinned = vision & enemies;
        if possibly_pinned != 0 {
            pinners = self.mg.b_moves( opp_king_pos, occupancy_wo_opp_king ^ possibly_pinned ) &
                      ( self.bit_board[ side | QUEEN ] | self.bit_board[ side | BISHOP ] );
            while pinners != 0 {
                pos = pop_lsb_pos( &mut pinners );
                pin = line_segment( pos, opp_king_pos ) ^ opp_king;
                pinned_pos = ( possibly_pinned & pin ).trailing_zeros() as usize;
                self.a_pins[ pinned_pos ] &= pin;
            }
        }

        /* Orthogonal pins */

        // Our King
        vision = self.mg.r_moves( king_pos, occupancy_wo_king );
        possibly_pinned = vision & friends;
        if possibly_pinned != 0 {
            possibly_attacking = vision & enemies;
            pinners = self.mg.r_moves( king_pos, occupancy_wo_king ^ possibly_pinned ) &
                      ( ( self.bit_board[ opp_side | QUEEN ] | self.bit_board[ opp_side | ROOK ] ) & !possibly_attacking );
            while pinners != 0 {
                pos = pop_lsb_pos( &mut pinners );
                pin = line_segment( pos, king_pos ) ^ king;
                pinned_pos = ( possibly_pinned & pin ).trailing_zeros() as usize;
                self.a_pins[ pinned_pos ] &= pin;
            }
        }

        // Enemy King
        vision = self.mg.r_moves( opp_king_pos, occupancy_wo_opp_king );
        possibly_pinned = vision & enemies;
        if possibly_pinned != 0 {
            pinners = self.mg.r_moves( opp_king_pos, occupancy_wo_opp_king ^ possibly_pinned ) &
                      ( self.bit_board[ opp_side | QUEEN ] | self.bit_board[ opp_side | ROOK ] );
            while pinners != 0 {
                pos = pop_lsb_pos( &mut pinners );
                pin = line_segment( pos, opp_king_pos ) ^ opp_king;
                pinned_pos = ( possibly_pinned & pin ).trailing_zeros() as usize;
                self.a_pins[ pinned_pos ] &= pin;
            }
        }

        // ep pins - the special stuff - can only happen to us (it's our move)
        if self.ep_flag() {
            let mut ep_killers = self.bit_board[ side | PAWN ] & self.mg.p_captures( self.en_passant, opp_side );
            if ep_killers != 0 {
                self.ep_possible = true; // tentatively
                let ep_target = self.ep_target();
                let ep_bb = self.ep_bb();
                let ep_target_bb: u64 = 1 << ep_target;

                // Everything except EP, basically if EP capture leads to check, ban it!
                pin = FULL_BOARD ^ ep_bb;

                // Check if ep_target is diagonally pinned to our king
                // Btw, it cannot be pinned orthogonally (unless both ep_killer and ep_target are horizontally pinned to our king, which is handled after this)
                let mut ep_diag_pin = false;
                vision = self.mg.b_moves( king_pos, occupancy_wo_king );
                possibly_pinned = vision & ep_target_bb;
                if possibly_pinned != 0 {
                    possibly_attacking = vision & enemies;
                    pinners = self.mg.b_moves( king_pos, occupancy_wo_king ^ possibly_pinned ) &
                              ( ( self.bit_board[ opp_side | QUEEN ] | self.bit_board[ opp_side | BISHOP ] ) & !possibly_attacking );
                    if pinners != 0 { ep_diag_pin = true; }
                }

                if ep_diag_pin {
                    self.ep_possible = false;
                    while ep_killers != 0 {
                        pinned_pos = pop_lsb_pos( &mut ep_killers );
                        self.a_pins[ pinned_pos ] &= pin; // Everything except EP
                    }
                } else if king_pos / 8 == ep_target / 8 {
                    // Check if the capturing pawn(s) are horizontally pinned to our king (can only be horizontal...)
                    while ep_killers != 0 {
                        pinned_pos = pop_lsb_pos( &mut ep_killers );
                        let ep_apply: u64 = ( 1 << pinned_pos ) | ep_target_bb | ep_bb;
                        vision = self.mg.r_moves( king_pos, occupancy_wo_king );
                        if vision & ep_apply != 0 {
                            possibly_attacking = vision & enemies;
                            pinners = self.mg.r_moves( king_pos, occupancy_wo_king ^ ep_apply ) &
                                      ( ( self.bit_board[ opp_side | QUEEN ] | self.bit_board[ opp_side | ROOK ] ) & !possibly_attacking );

                            if pinners != 0 {
                                self.ep_possible = false;
                                self.a_pins[ pinned_pos ] &= pin;
                            }
                        }
                    }
                }

                if self.ep_possible {
                    // No ep pins... let's see if the king is in check...
                    if self.num_checks > 0 {
                        // Can only be exactly ONE check
                        // Two possibilities -
                        // 1. Discovered check - CAN'T EP!
                        // 2. The pawn that just moved is checking the king - CAN EP!

                        if self.check_blocker & ep_target_bb == 0 {
                            // No. 1
                            self.ep_possible = false;
                        }
                    } else {
                        // No ep pins AND king is not in check (check_blocker == FULL_BOARD)
                        // Let's check for natural pins that may be blocking ep capture
                        let mut ep_killers = self.bit_board[ side | PAWN ] & self.mg.p_captures( self.en_passant, opp_side );
                        let mut nat_pin: bool = true;
                        while ep_killers != 0 {
                            pos = pop_lsb_pos( &mut ep_killers );
                            nat_pin = nat_pin && ( self.a_pins[ pos ] & ep_bb == 0 );
                        }

                        self.ep_possible = !nat_pin;
                    }
                }
            }
        }
    }

    pub fn is_legal( &self, mv: &Move ) -> bool {
        let castling_path = mv.castling_path(); // zero if not castling
        if castling_path != 0 {
            self.attacked & castling_path == 0 // No checks on the king's path incluing the starting and ending square
        } else {
            if mv.piece == ( self.to_move | KING ) {
                self.attacked & ( 1 << mv.to ) == 0 // King can't move into check
            } else if self.num_checks > 1 {
                false // Double check, only the King can move
            } else {
                if mv.piece == ( self.to_move | PAWN ) && self.ep_flag() && ( self.check_blocker & self.ep_target_bb() != 0 ) {
                    // Enemy pawn checking our king can be captured en passant by my pawns
                    ( ( self.check_blocker | self.ep_bb() ) & self.a_pins[ mv.from ] ) & ( 1 << mv.to ) != 0 // The move shouldn't break out of an a_pin and should block check, if any
                } else {
                    ( self.check_blocker & self.a_pins[ mv.from ] ) & ( 1 << mv.to ) != 0 // The move shouldn't break out of an a_pin and should block check, if any
                }
            }
        }
    }

    pub fn legal_moves( &self ) -> Vec<Move> {
        let moves = self.moves();

        // Hmm, this is faster than both (by ~25%) -
        // 1. legal_moves = moves.into_iter().filter( |x| self.is_legal( &x ) ).collect();
        // 2. legal_moves = moves.iter().filter( |x| self.is_legal( x ) ).map( |x| *x ).collect();
        let mut legal_moves: Vec<Move> = Vec::new();
        for mv in &moves {
            if self.is_legal( mv ) { legal_moves.push( *mv ); }
        }

        legal_moves
    }

    pub fn perft( &mut self, depth: usize, divide: bool ) -> u64 {
        assert!( depth > 0, "Depth has to be greater than zero!" );

        let legal_moves = self.legal_moves();

        if depth == 1 {
            legal_moves.len() as u64
        } else {
            let mut nodes: u64 = 0;
            let mut nodes_child: u64;
            let irs = self.ir_state();

            for mv in &legal_moves {
                self.make( mv );
                nodes_child = self.perft( depth - 1, false );
                self.unmake( mv, &irs );

                if divide {
                    println!( "{}{}: {}", offset_to_algebraic( mv.from ), offset_to_algebraic( mv.to ), nodes_child );
                }

                nodes += nodes_child;
            }

            nodes
        }
    }

    pub fn set_hash( &mut self ) {
        self.hash = 0;

        // to_move
        if self.to_move == WHITE { self.hash ^= self.hg.side_hash; }

        // pieces
        for ( pos, piece ) in self.simple_board.iter().enumerate() {
            if *piece != EMPTY {
                self.hash ^= self.hg.piece( *piece, pos );
            }
        }

        // castling
        self.hash ^= self.hg.castling( self.castling );

        // ep
        if self.ep_possible { self.hash ^= self.hg.ep( self.en_passant ); }
    }

    // Asserts that Incrementally computed hash is same as the one computed from scratch
    // true = OK
    pub fn check_hash( &mut self ) -> bool {
        let hash = self.hash;
        self.set_hash();
        hash == self.hash
    }

    // Recursively check_hash till the given depth
    pub fn check_hash_rec( &mut self, depth: usize ) -> bool {
        assert!( depth > 0, "Depth has to be greater than zero!" );

        let legal_moves = self.legal_moves();

        if depth == 1 {
            self.check_hash()
        } else {
            let mut ok: bool = true;
            let irs = self.ir_state();

            for mv in &legal_moves {
                self.make( mv );
                ok = ok && self.check_hash_rec( depth - 1 );
                self.unmake( mv, &irs );
            }

            ok
        }
    }

    pub fn hash_perft( &mut self, depth: usize, divide: bool ) -> u64 {
        // Initialize HashTable
        let num_bits = if depth < 7 {
            20
        } else if depth == 7 {
            24
        } else {
            26
        };

        let mut hp = HashPerft { ..Default::default() };
        hp.init( num_bits );

        self.hash_perft_rep( depth, divide, &mut hp )
    }

    pub fn hash_perft_rep( &mut self, depth: usize, divide: bool, hp: &mut HashPerft ) -> u64 {
        assert!( depth > 0, "Depth has to be greater than zero!" );

        if let Some( nodes ) = hp.get( self.hash, depth ) {
            nodes
        } else {
            let legal_moves = self.legal_moves();

            if depth == 1 {
                legal_moves.len() as u64
            } else {
                let mut nodes: u64 = 0;
                let mut nodes_child: u64;
                let irs = self.ir_state();

                for mv in &legal_moves {
                    self.make( mv );
                    nodes_child = self.hash_perft_rep( depth - 1, false, hp );
                    self.unmake( mv, &irs );

                    if divide {
                        println!( "{}{}: {}", offset_to_algebraic( mv.from ), offset_to_algebraic( mv.to ), nodes_child );
                    }

                    nodes += nodes_child;
                }

                // Store in hash table
                hp.set( self.hash, depth, nodes );

                nodes
            }
        }
    }
}
