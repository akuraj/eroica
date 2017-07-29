//! State representation: Define required types and functions to construct and store the state

use std::ops::{ Index, IndexMut };
use std::fmt;
use consts::*;
use utils::*;
use movegen::*;
use hash::*;
use hashtables::*;
use std::collections::VecDeque;
use std::cmp;
use std::cmp::Ordering;

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
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct Move {
    pub piece: u8,
    pub from: usize,
    pub to: usize,
    pub capture: u8,
    pub promotion: u8,
    pub see: i32,
    pub pst_eval: PSTEval,
}

impl Move {
    // Returns ( castling_identifier, castling_path )
    pub fn castling_info( &self ) -> ( u8, u64 ) {
        match ( self.piece, self.from ) {
            ( WHITE_KING, WK_START ) => match self.to {
                WK_QS_CASTLE => ( WQ_CASTLE, WQCR ),
                WK_KS_CASTLE => ( WK_CASTLE, WKCR ),
                _ => ( 0, 0 ),
            },
            ( BLACK_KING, BK_START ) => match self.to {
                BK_QS_CASTLE => ( BQ_CASTLE, BQCR ),
                BK_KS_CASTLE => ( BK_CASTLE, BKCR ),
                _ => ( 0, 0 ),
            },
            _ => ( 0, 0 ),
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

    pub fn null_move( piece: u8, pos: usize ) -> Self {
        Move { piece: piece,
               from: pos,
               to: ERR_POS,
               capture: EMPTY,
               promotion: EMPTY,
               see: 0,
               pst_eval: PSTEval::new(), }
    }

    // Score: Tapered Evaluation using PST + SEE
    #[inline]
    pub fn score( &self ) -> i32 {
        self.pst_eval.tapered_eval( self.piece & COLOR ) + self.see
    }
}

impl Ord for Move {
    fn cmp( &self, other: &Move ) -> Ordering {
        other.score().cmp( &self.score() )
    }
}

impl PartialOrd for Move {
    fn partial_cmp( &self, other: &Move ) -> Option<Ordering> {
        Some( self.cmp( other ) )
    }
}

impl fmt::Display for Move {
    fn fmt( &self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        let castling = self.castling_info().0;
        if castling != 0 {
            if castling == WK_CASTLE || castling == BK_CASTLE {
                output.push_str( "O-O" );
            } else if castling == WQ_CASTLE || castling == BQ_CASTLE {
                output.push_str( "O-O-O" );
            } else {
                panic!( "Invalid Castling: {}!", castling );
            }
        } else {
            output.push_str( &( offset_to_algebraic( self.from ) ) );
            if self.capture != EMPTY {
                output.push( 'x' );
            } else {
                output.push( '-' );
            }
            output.push_str( &( offset_to_algebraic( self.to ) ) );
            if self.promotion != EMPTY {
                output.push( '=' );
                output.push( piece_to_char( self.promotion ) );
            }
        }

        write!( f, "{}", output )
    }
}

// PSTEval: Stores info required to compute Tapered Eval
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct PSTEval {
    pub npm: i32,
    pub eval_mg: i32,
    pub eval_eg: i32,
}

impl PSTEval {
    pub fn new() -> Self {
        PSTEval {
            npm: 0,
            eval_mg: 0,
            eval_eg: 0,
        }
    }

    #[inline]
    pub fn tapered_eval( &self, to_move: u8 ) -> i32 {
        // Tapered Eval from side-to-move's POV
        let phase = ( ( cmp::max( EG_NPM_LIMIT, cmp::min( MG_NPM_LIMIT, self.npm ) ) - EG_NPM_LIMIT ) * MG_PHASE ) / ( MG_NPM_LIMIT - EG_NPM_LIMIT );
        let eval = ( phase * self.eval_mg + ( MG_PHASE - phase ) * self.eval_eg ) / MG_PHASE;
        TEMPO_BONUS + if to_move == WHITE { eval } else { -eval }
    }
}

// Game status enum
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum Status {
    /* Termination Trump: Checkmate > Stalemate > FiftyMoveDraw > RepetitionDraw > InsufficientMaterial */

    // Absolute
    Checkmate,
    Stalemate,

    // Various -Draw- situations
    FiftyMoveDraw,
    RepetitionDraw,
    InsufficientMaterial,

    Ongoing
}

// Irreversible State - and also Control info, which is expensive to recalc
pub struct IRState {
    // IRState
    pub castling: u8,
    pub en_passant: usize, // Store the pos (square address)
    pub halfmove_clock: usize,

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

    // PSTEval
    pub pst_eval: PSTEval,
}

// Full State
pub struct State {
    // State (the board) + IRState
    pub simple_board: SimpleBoard,
    pub bit_board: BitBoard,
    pub to_move: u8,
    pub castling: u8,
    pub en_passant: usize, // Store the pos (square address)
    pub halfmove_clock: usize,
    pub fullmove_count: usize, // Starts at 1 (First Move)

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

    // History
    pub history: VecDeque<u64>,

    // PSTEval
    pub pst_eval: PSTEval,
}

impl fmt::Display for State {
    fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
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
        let mut state = State { simple_board: [ EMPTY; 64 ],
                                bit_board: BitBoard( [ 0; 14 ] ),
                                to_move: 0,
                                castling: 0,
                                en_passant: ERR_POS,
                                halfmove_clock: 0,
                                fullmove_count: 0,
                                mg: MoveGen::new( true ),
                                attacked: 0,
                                num_checks: 0,
                                check_blocker: FULL_BOARD,
                                defended: 0,
                                a_pins: [ FULL_BOARD; 64 ],
                                control: [ 0; 64 ],
                                ep_possible: false,
                                hg: HashGen::new(),
                                hash: 0,
                                history: VecDeque::new(),
                                pst_eval: PSTEval::new(), };

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
                         _  => section.parse::<usize>().unwrap(),
                    }
                },
                5 => {
                    // fullmove_count
                    state.fullmove_count = match section {
                        "-" => 1,
                         _  => section.parse::<usize>().unwrap(),
                    }
                },
                _ => {},
            }
        }

        state.compute_control();
        state.state_check();
        state.set_hash();
        state.set_pst_eval();

        state
    }

    pub fn state_check( &self ) {
        // Very basic checks -> does not assert that the position is a child of START_FEN (can be reached from the initial position)
        // Sufficient for the engine to play

        // Check number of kings
        assert_eq!( self.bit_board[ WHITE_KING ].count_ones(), 1 );
        assert_eq!( self.bit_board[ BLACK_KING ].count_ones(), 1 );

        // Check that opposing king is not in check if it's our move
        assert_eq!( self.defended & self.bit_board[ ( self.to_move ^ COLOR ) | KING ], 0 );

        // castling
        if self.castling & W_CASTLE != 0 {
            assert_eq!( self.simple_board[ WK_START ], WHITE_KING );

            if self.castling & WK_CASTLE != 0 {
                assert_eq!( self.simple_board[ WKR_START ], WHITE_ROOK );
            }

            if self.castling & WQ_CASTLE != 0 {
                assert_eq!( self.simple_board[ WQR_START ], WHITE_ROOK );
            }
        }

        if self.castling & B_CASTLE != 0 {
            assert_eq!( self.simple_board[ BK_START ], BLACK_KING );

            if self.castling & BK_CASTLE != 0 {
                assert_eq!( self.simple_board[ BKR_START ], BLACK_ROOK );
            }

            if self.castling & BQ_CASTLE != 0 {
                assert_eq!( self.simple_board[ BQR_START ], BLACK_ROOK );
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

    pub fn fen( &self, strict_ep: bool ) -> String {
        let mut output = String::new();

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
        if self.castling & WK_CASTLE != 0 { output.push( 'K' ) }
        if self.castling & WQ_CASTLE != 0 { output.push( 'Q' ) }
        if self.castling & BK_CASTLE != 0 { output.push( 'k' ) }
        if self.castling & BQ_CASTLE != 0 { output.push( 'q' ) }
        if self.castling & ( W_CASTLE | B_CASTLE ) == 0 { output.push( '-' ) }
        output.push( ' ' );

        // ep
        if self.ep_flag() && ( !strict_ep || self.ep_possible ) {
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

        output
    }

    pub fn new() -> Self {
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
        self.pst_eval = irs.pst_eval;
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
                 hash: self.hash,
                 pst_eval: self.pst_eval, }
    }

    pub fn make( &mut self, mv: &Move ) {
        // Add current hash to history
        self.history.push_front( self.hash );

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
                match mv.castling_info().0 {
                    WK_CASTLE => {
                        self.simple_board[ WKR_START ] = EMPTY;
                        self.simple_board[ WKR_CASTLE ] = WHITE_ROOK;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WKC;
                        self.hash ^= self.hg.piece( WHITE_ROOK, WKR_START ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( WHITE_ROOK, WKR_CASTLE ); // HASH_UPDATE
                    },
                    WQ_CASTLE => {
                        self.simple_board[ WQR_START ] = EMPTY;
                        self.simple_board[ WQR_CASTLE ] = WHITE_ROOK;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WQC;
                        self.hash ^= self.hg.piece( WHITE_ROOK, WQR_START ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( WHITE_ROOK, WQR_CASTLE ); // HASH_UPDATE
                    },
                    _ => {},
                };

                self.castling &= !W_CASTLE;
            },
            WHITE_ROOK => {
                match mv.from {
                    WQR_START => self.castling &= !WQ_CASTLE,
                    WKR_START => self.castling &= !WK_CASTLE,
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
                match mv.castling_info().0 {
                    BK_CASTLE => {
                        self.simple_board[ BKR_START ] = EMPTY;
                        self.simple_board[ BKR_CASTLE ] = BLACK_ROOK;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BKC;
                        self.hash ^= self.hg.piece( BLACK_ROOK, BKR_START ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( BLACK_ROOK, BKR_CASTLE ); // HASH_UPDATE
                    },
                    BQ_CASTLE => {
                        self.simple_board[ BQR_START ] = EMPTY;
                        self.simple_board[ BQR_CASTLE ] = BLACK_ROOK;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BQC;
                        self.hash ^= self.hg.piece( BLACK_ROOK, BQR_START ); // HASH_UPDATE
                        self.hash ^= self.hg.piece( BLACK_ROOK, BQR_CASTLE ); // HASH_UPDATE
                    },
                    _ => {},
                };

                self.castling &= !B_CASTLE;
            },
            BLACK_ROOK => {
                match mv.from {
                    BQR_START => self.castling &= !BQ_CASTLE,
                    BKR_START => self.castling &= !BK_CASTLE,
                    _ => {},
                }
            },
            _ => {},
        }

        // Rook Capture (update Castling)
        match mv.capture {
            WHITE_ROOK => {
                match mv.to {
                    WQR_START => self.castling &= !WQ_CASTLE,
                    WKR_START => self.castling &= !WK_CASTLE,
                    _ => {},
                }
            },
            BLACK_ROOK => {
                match mv.to {
                    BQR_START => self.castling &= !BQ_CASTLE,
                    BKR_START => self.castling &= !BK_CASTLE,
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

        // Copy over PSTEval from mv
        self.pst_eval = mv.pst_eval;
    }

    pub fn unmake( &mut self, mv: &Move, irs: &IRState ) {
        // Pop the current hash
        self.history.pop_front();

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
                match mv.castling_info().0 {
                    WK_CASTLE => {
                        self.simple_board[ WKR_START ] = WHITE_ROOK;
                        self.simple_board[ WKR_CASTLE ] = EMPTY;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WKC;
                    },
                    WQ_CASTLE => {
                        self.simple_board[ WQR_START ] = WHITE_ROOK;
                        self.simple_board[ WQR_CASTLE ] = EMPTY;
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
                match mv.castling_info().0 {
                    BK_CASTLE => {
                        self.simple_board[ BKR_START ] = BLACK_ROOK;
                        self.simple_board[ BKR_CASTLE ] = EMPTY;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BKC;
                    },
                    BQ_CASTLE => {
                        self.simple_board[ BQR_START ] = BLACK_ROOK;
                        self.simple_board[ BQR_CASTLE ] = EMPTY;
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

    // All pseudo-legal moves for a given piece at a given pos, added to moves
    pub fn add_moves_from_bb( &self, piece: u8, pos: usize, moves_bb: &mut u64, moves: &mut Vec<Move> ) {
        let mut mv = Move::null_move( piece, pos );
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

        // PAWN
        piece = side | PAWN;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.p_moves( pos, side, occupancy_w_ep ) & not_friendly;
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

        // BISHOP
        piece = side | BISHOP;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.b_moves( pos, occupancy ) & not_friendly;
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

        // QUEEN
        piece = side | QUEEN;
        bb = self.bit_board[ piece ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            moves_bb = self.mg.q_moves( pos, occupancy ) & not_friendly;
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
        let mut attacks: u64;

        /**** Enemies ****/

        // ROOK & QUEEN - orthogonal attacks
        bb = self.bit_board[ opp_side | ROOK ] | self.bit_board[ opp_side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.r_moves( pos, occupancy_wo_king );
            self.control[ pos ] |= attacks;
            self.attacked |= attacks;

            if attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= line_segment( pos, king_pos ) ^ king;
            }
        }

        // BISHOP & QUEEN - diagonal attacks
        bb = self.bit_board[ opp_side | BISHOP ] | self.bit_board[ opp_side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.b_moves( pos, occupancy_wo_king );
            self.control[ pos ] |= attacks;
            self.attacked |= attacks;

            if attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= line_segment( pos, king_pos ) ^ king;
            }
        }

        // KNIGHT
        bb = self.bit_board[ opp_side | KNIGHT ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.n_moves( pos );
            self.control[ pos ] |= attacks;
            self.attacked |= attacks;

            if attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= 1 << pos;
            }
        }

        // PAWN
        bb = self.bit_board[ opp_side | PAWN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.p_captures( pos, opp_side );
            self.control[ pos ] |= attacks;
            self.attacked |= attacks;

            if attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= 1 << pos;
            }
        }

        // KING
        bb = self.bit_board[ opp_side | KING ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.k_captures( pos );
            self.control[ pos ] |= attacks;
            self.attacked |= attacks;
        }

        /**** Friends ****/

        // ROOK & QUEEN - orthogonal attacks
        bb = self.bit_board[ side | ROOK ] | self.bit_board[ side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.r_moves( pos, occupancy );
            self.control[ pos ] |= attacks;
            self.defended |= attacks;
        }

        // BISHOP & QUEEN - diagonal attacks
        bb = self.bit_board[ side | BISHOP ] | self.bit_board[ side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.b_moves( pos, occupancy );
            self.control[ pos ] |= attacks;
            self.defended |= attacks;
        }

        // KNIGHT
        bb = self.bit_board[ side | KNIGHT ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.n_moves( pos );
            self.control[ pos ] |= attacks;
            self.defended |= attacks;
        }

        // PAWN
        bb = self.bit_board[ side | PAWN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.p_captures( pos, side );
            self.control[ pos ] |= attacks;
            self.defended |= attacks;
        }

        // KING
        bb = self.bit_board[ side | KING ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            attacks = self.mg.k_captures( pos );
            self.control[ pos ] |= attacks;
            self.defended |= attacks;
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

    // For pseudo-legal moves
    pub fn is_legal( &self, mv: &Move ) -> bool {
        let castling_path = mv.castling_info().1; // zero if not castling
        if castling_path != 0 {
            self.attacked & castling_path == 0 // No checks on the king's path including the starting and ending square
        } else {
            if mv.piece == ( self.to_move | KING ) {
                self.attacked & ( 1 << mv.to ) == 0 // The King can't move into check
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

    // All possible pseudo-legal moves for piece at pos
    pub fn moves_bb( &self, pos: usize ) -> u64 {
        let side = self.to_move;
        let piece = self.simple_board[ pos ];
        if piece == EMPTY || piece & COLOR != side { return 0; }

        let not_friendly = !self.bit_board[ side | ALL ];
        let occupancy = self.bit_board[ side | ALL ] | self.bit_board[ ( side ^ COLOR ) | ALL ];

        not_friendly & ( match piece & COLOR_MASK {
            PAWN => self.mg.p_moves( pos, side, occupancy | self.ep_bb() ),
            KNIGHT => self.mg.n_moves( pos ),
            BISHOP => self.mg.b_moves( pos, occupancy ),
            ROOK => self.mg.r_moves( pos, occupancy ),
            QUEEN => self.mg.q_moves( pos, occupancy ),
            KING => self.mg.k_moves( pos, side, occupancy, self.castling ),
            _ => panic!( "Invalid piece: {}", piece ),
        } )
    }

    // For any move
    pub fn is_legal_strict( &self, mv: &Move ) -> bool {
        if self.simple_board[ mv.from ] != mv.piece || self.simple_board[ mv.to ] != mv.capture { return false; }
        if self.moves_bb( mv.from ) & ( 1 << mv.to ) == 0 { return false; }
        self.is_legal( mv )
    }

    #[inline]
    pub fn num_repetitions( &self, depth: usize ) -> usize {
        self.history.iter().take( depth ).enumerate().filter( |x| x.0 % 2 == 1 && *x.1 == self.hash ).count()
    }

    // This function returns legal Moves (sorted by PSTEval + SEE) and Game Status
    pub fn node_info( &self ) -> ( Vec<Move>, Status ) {
        let mut moves = self.moves();

        // Hmm, this is faster than both (by ~25%) -
        // 1. legal_moves = moves.into_iter().filter( |x| self.is_legal( &x ) ).collect();
        // 2. legal_moves = moves.iter().filter( |x| self.is_legal( x ) ).map( |x| *x ).collect();
        let mut legal_moves: Vec<Move> = Vec::new();
        for mv in moves.iter_mut() {
            if self.is_legal( mv ) {
                mv.see = self.see( mv );
                self.incremental_pst_eval( mv );
                legal_moves.push( *mv );
            }
        }

        // Sort by PSTEval + SEE
        legal_moves.sort();

        // compute status and return
        if legal_moves.len() == 0 {
            if self.num_checks > 0 {
                ( legal_moves, Status::Checkmate )
            } else {
                ( legal_moves, Status::Stalemate )
            }
        } else if self.halfmove_clock > 99 {
            ( legal_moves, Status::FiftyMoveDraw )
        } else {
            let rev_history = cmp::min( self.halfmove_clock as usize, self.history.len() ); // Available reversible history
            if rev_history > 7 && self.num_repetitions( rev_history ) > 1 {
                ( legal_moves, Status::RepetitionDraw )
            } else {
                // p_n_p = pieces and pawns
                let p_n_p = ( self.bit_board[ WHITE_KING ] | self.bit_board[ BLACK_KING ] ) ^ ( self.bit_board[ WHITE_ALL ] | self.bit_board[ BLACK_ALL ] );
                match p_n_p.count_ones() {
                    0 => ( legal_moves, Status::InsufficientMaterial ),
                    1 => {
                        let survivor = self.simple_board[ p_n_p.trailing_zeros() as usize ] & COLOR_MASK;
                        if survivor == BISHOP || survivor == KNIGHT {
                            ( legal_moves, Status::InsufficientMaterial )
                        } else {
                            ( legal_moves, Status::Ongoing )
                        }
                    },
                    _ => ( legal_moves, Status::Ongoing )
                }
            }
        }
    }

    // Legal Moves, unsorted
    pub fn legal_moves( &self ) -> Vec<Move> {
        let mut moves = self.moves();

        // Hmm, this is faster than both (by ~25%) -
        // 1. legal_moves = moves.into_iter().filter( |x| self.is_legal( &x ) ).collect();
        // 2. legal_moves = moves.iter().filter( |x| self.is_legal( x ) ).map( |x| *x ).collect();
        let mut legal_moves: Vec<Move> = Vec::new();
        for mv in moves.iter_mut() {
            if self.is_legal( mv ) {
                legal_moves.push( *mv );
            }
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

        let mut hp = HashPerft::new( num_bits );

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

    pub fn make_moves( &mut self, mvs: &[ Move ] ) {
        for mv in mvs {
            if self.is_legal_strict( mv ) {
                self.make( mv );
            } else {
                panic!( "Illegal move: {:?}", mv );
            }
        }
    }

    pub fn is_tactical( &self, mv: &Move ) -> bool {
        mv.capture != EMPTY || mv.promotion != EMPTY
    }

    pub fn tactical_moves( &self, legal_moves: &[ Move ] ) -> Vec<Move> {
        let mut tactical_moves: Vec<Move> = Vec::new();
        for mv in legal_moves {
            if self.is_tactical( mv ) { tactical_moves.push( *mv ); }
        }

        tactical_moves
    }

    // Attackers of a given square
    pub fn attackers( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.mg.p_captures( pos, BLACK ) & self.bit_board[ WHITE_PAWN ] |
        self.mg.p_captures( pos, WHITE ) & self.bit_board[ BLACK_PAWN ] |
        self.mg.n_moves( pos ) & ( self.bit_board[ WHITE_KNIGHT ] | self.bit_board[ BLACK_KNIGHT ] ) |
        self.mg.b_moves( pos, occupancy ) & ( self.bit_board[ WHITE_BISHOP ] | self.bit_board[ BLACK_BISHOP ] | self.bit_board[ WHITE_QUEEN ] | self.bit_board[ BLACK_QUEEN ] ) |
        self.mg.r_moves( pos, occupancy ) & ( self.bit_board[ WHITE_ROOK ] | self.bit_board[ BLACK_ROOK ] | self.bit_board[ WHITE_QUEEN ] | self.bit_board[ BLACK_QUEEN ] ) |
        self.mg.k_captures( pos ) & ( self.bit_board[ WHITE_KING ] | self.bit_board[ BLACK_KING ] )
    }

    // Returns the least valuable attacker and updates the attackers and occupancy bit_boards. Also checks for x-ray attacks.
    pub fn min_attacker( &self, to_move: u8, piece_type: u8, pos: usize, stm_attackers: u64, occupancy: &mut u64, attackers: &mut u64 ) -> u8 {
        if piece_type == KING {
            KING // No need to update the bit_boards here, this is the end of the find-victim cycle
        } else {
            let piece_bb = stm_attackers & self.bit_board[ to_move | piece_type ];
            if piece_bb == 0 {
                self.min_attacker( to_move, piece_type + 2, pos, stm_attackers, occupancy, attackers ) // piece_type + 2 gives you the successor. P->N->B->R->Q->K
            } else {
                *occupancy ^= piece_bb & 0u64.wrapping_sub( piece_bb );

                // Diagonal x-ray
                if piece_type == PAWN || piece_type == BISHOP || piece_type == QUEEN {
                    *attackers |= self.mg.b_moves( pos, *occupancy ) & ( self.bit_board[ WHITE_BISHOP ] |
                                                                         self.bit_board[ BLACK_BISHOP ] |
                                                                         self.bit_board[ WHITE_QUEEN ] |
                                                                         self.bit_board[ BLACK_QUEEN ] );
                }

                // Orthogonal x-ray
                if piece_type == ROOK || piece_type == QUEEN {
                    *attackers |= self.mg.b_moves( pos, *occupancy ) & ( self.bit_board[ WHITE_ROOK ] |
                                                                         self.bit_board[ BLACK_ROOK ] |
                                                                         self.bit_board[ WHITE_QUEEN ] |
                                                                         self.bit_board[ BLACK_QUEEN ] );
                }

                *attackers &= *occupancy;

                piece_type
            }
        }
    }

    // Static Exchange Evaluation (SEE)
    // NOTE: The first ply is forced and will be scored by the static evaluation (pst_eval), and will not be counted in the SEE
    pub fn see( &self, mv: &Move ) -> i32 {
        let mut to_move = mv.piece & COLOR;

        // Castling will have an SEE of 0
        if mv.castling_info().0 != 0 { return 0; }

        let from = mv.from;
        let to = mv.to;
        let mut occupancy: u64 = ( self.bit_board[ WHITE_ALL ] | self.bit_board[ BLACK_ALL ] ) ^ ( 1 << from );

        // en_passant
        if self.en_passant == mv.to && mv.piece & COLOR_MASK == PAWN { occupancy ^= self.ep_target_bb(); }

        // If we have nobody defending the target square, we are done.
        let mut attackers = self.attackers( to, occupancy ) & occupancy;
        to_move ^= COLOR;
        let mut stm_attackers = attackers & self.bit_board[ to_move | ALL ];
        if stm_attackers == 0 { return 0; }

        // swap_list; swap_list[ 0 ] is going to remain zero (read function description)
        let mut swap_list: [ i32; 32 ] = [ 0; 32 ];

        // Update next_victim if we have a promotion
        let mut next_victim = if mv.promotion == EMPTY { mv.piece & COLOR_MASK } else { mv.promotion & COLOR_MASK };
        let mut index: usize = 0;

        while stm_attackers != 0 {
            index += 1;

            swap_list[ index ] = -swap_list[ index - 1 ] + piece_value_mg( next_victim );
            next_victim = self.min_attacker( to_move, PAWN, to, stm_attackers, &mut occupancy, &mut attackers );

            if next_victim == KING {
                if stm_attackers != attackers { index -= 1; } // Can't capture a defended piece with the King
                break; // the end of the find-victim cycle
            }

            to_move ^= COLOR;
            stm_attackers = attackers & self.bit_board[ to_move | ALL ];
        }

        while index > 0 {
            swap_list[ index - 1 ] = cmp::min( -swap_list[ index ], swap_list[ index - 1 ] );
            index -= 1;
        }

        swap_list[ 0 ]
    }

    // Calculate PSTEval from scratch and set
    pub fn set_pst_eval( &mut self ) {
        let pawn_pst: &[ i32 ] = &PAWN_PST;
        let knight_pst: &[ i32 ] = &KNIGHT_PST;
        let bishop_pst: &[ i32 ] = &BISHOP_PST;
        let rook_pst: &[ i32 ] = &ROOK_PST;
        let queen_pst: &[ i32 ] = &QUEEN_PST;

        let mut npm: i32 = 0;
        let mut eval_mg: i32 = 0;
        let mut eval_eg: i32 = 0;

        let mut piece_type: u8;
        let mut color: u8;
        let mut bb: u64;
        let mut pos: usize;

        for piece in ALL_PIECE_TYPES.iter() {
            piece_type = *piece & COLOR_MASK;
            color = *piece & COLOR;

            let ( piece_val_mg, piece_val_eg, pst ) : ( i32, i32, &[ i32 ] ) = match piece_type {
                PAWN => ( PAWN_VALUE_MG, PAWN_VALUE_EG, pawn_pst ),
                KNIGHT => ( KNIGHT_VALUE_MG, KNIGHT_VALUE_EG, knight_pst ),
                BISHOP => ( BISHOP_VALUE_MG, BISHOP_VALUE_EG, bishop_pst ),
                ROOK => ( ROOK_VALUE_MG, ROOK_VALUE_EG, rook_pst ),
                QUEEN => ( QUEEN_VALUE_MG, QUEEN_VALUE_EG, queen_pst ),
                _ => panic!( "Invalid piece type: {}", piece_type ),
            };

            bb = self.bit_board[ *piece ];
            match color {
                WHITE => {
                    while bb != 0 {
                        if piece_type != PAWN { npm += piece_val_mg; }
                        pos = pop_lsb_pos( &mut bb );
                        eval_mg += piece_val_mg + pst[ MAX_POS - pos ];
                        eval_eg += piece_val_eg + pst[ MAX_POS - pos ];
                    }
                },
                BLACK => {
                    while bb != 0 {
                        if piece_type != PAWN { npm += piece_val_mg; }
                        pos = pop_lsb_pos( &mut bb );
                        eval_mg -= piece_val_mg + pst[ pos ];
                        eval_eg -= piece_val_eg + pst[ pos ];
                    }
                },
                _ => panic!( "Invalid color: {}", color ),
            }
        }

        // Bishop Pair Bonus
        let bishop_pair_bonus = ( has_opp_color_pair( self.bit_board[ WHITE_BISHOP ] ) - has_opp_color_pair( self.bit_board[ BLACK_BISHOP ] ) ) * BISHOP_PAIR_BONUS;
        eval_mg += bishop_pair_bonus;
        eval_eg += bishop_pair_bonus;

        // Kings
        bb = self.bit_board[ WHITE_KING ];
        pos = pop_lsb_pos( &mut bb );
        eval_mg += KING_MG_PST[ MAX_POS - pos ];
        eval_eg += KING_EG_PST[ MAX_POS - pos ];

        bb = self.bit_board[ BLACK_KING ];
        pos = pop_lsb_pos( &mut bb );
        eval_mg -= KING_MG_PST[ pos ];
        eval_eg -= KING_EG_PST[ pos ];

        self.pst_eval.npm = npm;
        self.pst_eval.eval_mg = eval_mg;
        self.pst_eval.eval_eg = eval_eg;
    }

    // Tapered Evaluation using PST
    pub fn pst_tapered_eval( &self ) -> i32 {
        self.pst_eval.tapered_eval( self.to_move )
    }

    pub fn incremental_pst_eval( &self, mv: &mut Move ) {
        let pawn_pst: &[ i32 ] = &PAWN_PST;
        let knight_pst: &[ i32 ] = &KNIGHT_PST;
        let bishop_pst: &[ i32 ] = &BISHOP_PST;
        let rook_pst: &[ i32 ] = &ROOK_PST;
        let queen_pst: &[ i32 ] = &QUEEN_PST;

        let mut d_npm: i32 = 0;
        let mut d_eval_mg: i32 = 0;
        let mut d_eval_eg: i32 = 0;

        let mut bishop_bb_w = self.bit_board[ WHITE_BISHOP ];
        let mut bishop_bb_b = self.bit_board[ BLACK_BISHOP ];
        let mut d_bishop_pair_bonus = -( has_opp_color_pair( bishop_bb_w ) - has_opp_color_pair( bishop_bb_b ) ) * BISHOP_PAIR_BONUS;

        // Mover
        match mv.piece {
            WHITE_KING => {
                d_eval_mg += KING_MG_PST[ MAX_POS - mv.to ] - KING_MG_PST[ MAX_POS - mv.from ];
                d_eval_eg += KING_EG_PST[ MAX_POS - mv.to ] - KING_EG_PST[ MAX_POS - mv.from ];
            },
            BLACK_KING => {
                d_eval_mg -= KING_MG_PST[ mv.to ] - KING_MG_PST[ mv.from ];
                d_eval_eg -= KING_EG_PST[ mv.to ] - KING_EG_PST[ mv.from ];
            },
            _ => {
                let pst: &[ i32 ] = match mv.piece & COLOR_MASK {
                    PAWN => pawn_pst,
                    KNIGHT => knight_pst,
                    BISHOP => bishop_pst,
                    ROOK => rook_pst,
                    QUEEN => queen_pst,
                    _ => panic!( "Invalid piece type: {}", mv.piece & COLOR_MASK ),
                };

                match mv.piece & COLOR {
                    WHITE => {
                        d_eval_mg += pst[ MAX_POS - mv.to ] - pst[ MAX_POS - mv.from ];
                        d_eval_eg += pst[ MAX_POS - mv.to ] - pst[ MAX_POS - mv.from ];
                    },
                    BLACK => {
                        d_eval_mg -= pst[ mv.to ] - pst[ mv.from ];
                        d_eval_eg -= pst[ mv.to ] - pst[ mv.from ];
                    },
                    _ => panic!( "Invalid color: {}", mv.piece & COLOR ),
                }
            },
        }

        // Capture
        if mv.capture != EMPTY {
            let ( piece_val_mg, piece_val_eg, pst ) : ( i32, i32, &[ i32 ] ) = match mv.capture & COLOR_MASK {
                PAWN => ( PAWN_VALUE_MG, PAWN_VALUE_EG, pawn_pst ),
                KNIGHT => ( KNIGHT_VALUE_MG, KNIGHT_VALUE_EG, knight_pst ),
                BISHOP => {
                    if mv.capture & COLOR == WHITE { bishop_bb_w ^= 1 << mv.to; } else { bishop_bb_b ^= 1 << mv.to; }
                    ( BISHOP_VALUE_MG, BISHOP_VALUE_EG, bishop_pst )
                },
                ROOK => ( ROOK_VALUE_MG, ROOK_VALUE_EG, rook_pst ),
                QUEEN => ( QUEEN_VALUE_MG, QUEEN_VALUE_EG, queen_pst ),
                _ => panic!( "Invalid piece type: {}", mv.capture & COLOR_MASK ),
            };

            match mv.capture & COLOR {
                WHITE => {
                    d_eval_mg -= piece_val_mg + pst[ MAX_POS - mv.to ];
                    d_eval_eg -= piece_val_eg + pst[ MAX_POS - mv.to ];
                },
                BLACK => {
                    d_eval_mg += piece_val_mg + pst[ mv.to ];
                    d_eval_eg += piece_val_eg + pst[ mv.to ];
                },
                _ => panic!( "Invalid color: {}", mv.capture & COLOR ),
            }

            if mv.capture & COLOR_MASK != PAWN {
                d_npm -= piece_val_mg;
            }
        }

        // Promotion
        if mv.is_promotion() {
            match mv.piece & COLOR {
                WHITE => {
                    d_eval_mg -= PAWN_VALUE_MG + PAWN_PST[ MAX_POS - mv.to ];
                    d_eval_eg -= PAWN_VALUE_EG + PAWN_PST[ MAX_POS - mv.to ];
                },
                BLACK => {
                    d_eval_mg += PAWN_VALUE_MG + PAWN_PST[ mv.to ];
                    d_eval_eg += PAWN_VALUE_EG + PAWN_PST[ mv.to ];
                },
                _ => panic!( "Invalid color: {}", mv.piece & COLOR ),
            }

            let ( piece_val_mg, piece_val_eg, pst ) : ( i32, i32, &[ i32 ] ) = match mv.promotion & COLOR_MASK {
                PAWN => ( PAWN_VALUE_MG, PAWN_VALUE_EG, pawn_pst ),
                KNIGHT => ( KNIGHT_VALUE_MG, KNIGHT_VALUE_EG, knight_pst ),
                BISHOP => {
                    if mv.promotion & COLOR == WHITE { bishop_bb_w ^= 1 << mv.to; } else { bishop_bb_b ^= 1 << mv.to; }
                    ( BISHOP_VALUE_MG, BISHOP_VALUE_EG, bishop_pst )
                },
                ROOK => ( ROOK_VALUE_MG, ROOK_VALUE_EG, rook_pst ),
                QUEEN => ( QUEEN_VALUE_MG, QUEEN_VALUE_EG, queen_pst ),
                _ => panic!( "Invalid piece type: {}", mv.promotion & COLOR_MASK ),
            };

            match mv.promotion & COLOR {
                WHITE => {
                    d_eval_mg += piece_val_mg + pst[ MAX_POS - mv.to ];
                    d_eval_eg += piece_val_eg + pst[ MAX_POS - mv.to ];
                },
                BLACK => {
                    d_eval_mg -= piece_val_mg + pst[ mv.to ];
                    d_eval_eg -= piece_val_eg + pst[ mv.to ];
                },
                _ => panic!( "Invalid color: {}", mv.promotion & COLOR ),
            }

            d_npm += piece_val_mg;
        }

        // En_passant
        if mv.piece & COLOR_MASK == PAWN && self.en_passant == mv.to {
            match mv.piece & COLOR {
                WHITE => {
                    let ep_target = mv.to - 8;
                    d_eval_mg += PAWN_VALUE_MG + PAWN_PST[ ep_target ];
                    d_eval_eg += PAWN_VALUE_EG + PAWN_PST[ ep_target ];
                },
                BLACK => {
                    let ep_target = mv.to + 8;
                    d_eval_mg -= PAWN_VALUE_MG + PAWN_PST[ MAX_POS - ep_target ];
                    d_eval_eg -= PAWN_VALUE_EG + PAWN_PST[ MAX_POS - ep_target ];
                },
                _ => panic!( "Invalid color: {}", mv.piece & COLOR ),
            }
        }

        // Castling
        match mv.castling_info().0 {
            WK_CASTLE => {
                d_eval_mg += ROOK_PST[ MAX_POS - WKR_CASTLE ] - ROOK_PST[ MAX_POS - WKR_START ];
                d_eval_eg += ROOK_PST[ MAX_POS - WKR_CASTLE ] - ROOK_PST[ MAX_POS - WKR_START ];
            },
            WQ_CASTLE => {
                d_eval_mg += ROOK_PST[ MAX_POS - WQR_CASTLE ] - ROOK_PST[ MAX_POS - WQR_START ];
                d_eval_eg += ROOK_PST[ MAX_POS - WQR_CASTLE ] - ROOK_PST[ MAX_POS - WQR_START ];
            },
            BK_CASTLE => {
                d_eval_mg -= ROOK_PST[ BKR_CASTLE ] - ROOK_PST[ BKR_START ];
                d_eval_eg -= ROOK_PST[ BKR_CASTLE ] - ROOK_PST[ BKR_START ];
            },
            BQ_CASTLE => {
                d_eval_mg -= ROOK_PST[ BQR_CASTLE ] - ROOK_PST[ BQR_START ];
                d_eval_eg -= ROOK_PST[ BQR_CASTLE ] - ROOK_PST[ BQR_START ];
            },
            _ => {},
        }

        d_bishop_pair_bonus += ( has_opp_color_pair( bishop_bb_w ) - has_opp_color_pair( bishop_bb_b ) ) * BISHOP_PAIR_BONUS;
        d_eval_mg += d_bishop_pair_bonus;
        d_eval_eg += d_bishop_pair_bonus;

        mv.pst_eval.npm = self.pst_eval.npm + d_npm;
        mv.pst_eval.eval_mg = self.pst_eval.eval_mg + d_eval_mg;
        mv.pst_eval.eval_eg = self.pst_eval.eval_eg + d_eval_eg;
    }

    // Asserts that Incrementally computed pst_eval is same as the one computed from scratch
    // true = OK
    pub fn check_pst_eval( &mut self ) -> bool {
        let pst_eval = self.pst_eval;
        self.set_pst_eval();
        pst_eval == self.pst_eval
    }

    pub fn check_pst_eval_rec( &mut self, depth: usize ) -> bool {
        assert!( depth > 0, "Depth has to be greater than zero!" );

        let ( legal_moves, _ ) = self.node_info();

        if depth == 1 {
            self.check_pst_eval()
        } else {
            let mut ok: bool = true;
            let irs = self.ir_state();

            for mv in &legal_moves {
                self.make( mv );
                ok = ok && self.check_pst_eval_rec( depth - 1 );
                self.unmake( mv, &irs );
            }

            ok
        }
    }

    pub fn evaluate_move( &self, mv: &mut Move ) {
        assert_eq!( self.to_move, mv.piece & COLOR );
        mv.see = self.see( mv );
        self.incremental_pst_eval( mv );
    }

    pub fn check_is_legal_strict_rec( &mut self, depth: usize ) -> bool {
        assert!( depth > 0, "Depth has to be greater than zero!" );

        let legal_moves = self.legal_moves();
        let mut ok: bool = true;

        if depth == 1 {
            for mv in &legal_moves {
                ok = ok && self.is_legal_strict( mv );
            }
        } else {
            let irs = self.ir_state();

            for mv in &legal_moves {
                ok = ok && self.is_legal_strict( mv );
                self.make( mv );
                ok = ok && self.check_is_legal_strict_rec( depth - 1 );
                self.unmake( mv, &irs );
            }
        }

        ok
    }
}
