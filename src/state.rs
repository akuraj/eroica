//! State representation: Define required types and functions to construct and store the state

use std::ops::{ Index, IndexMut };
use std::default::Default;
use std::fmt;
use consts::*;
use utils::*;
use movegen::*;

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
#[ derive( Copy, Clone, Debug, PartialEq ) ]
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

    pub fn is_ep( &self, en_passant: usize ) -> bool {
        ( self.piece & PAWN != 0 ) && ( self.to == en_passant )
    }
}

// Game status enum
#[ derive( Copy, Clone, Debug, PartialEq ) ]
pub enum Status {
    Unknown,
    Ongoing,
    Checkmate,
    Stalemate
}

// NodeInfo
#[ derive( Clone, Debug ) ]
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

// Irreversible State - and also attack and defend info - which is expensive to recalc
pub struct IRState {
    pub castling: u8,
    pub en_passant: usize, // Store the pos (square address)
    pub halfmove_clock: u8,

    // Expense stuff to recalc
    pub attacked: u64,
    pub num_checks: u8,
    pub check_blocker: u64,
    pub a_pins: [ u64; 64 ],
    pub node_info: NodeInfo,
}

// Full state
pub struct State {
    pub simple_board: SimpleBoard,
    pub bit_board: BitBoard,
    pub to_move: u8,
    pub castling: u8,
    pub en_passant: usize, // Store the pos (square address)
    pub halfmove_clock: u8,
    pub fullmove_count: u8, // Starts a 1 (First Move)

    pub mg: MoveGen,

    pub attacked: u64,
    pub num_checks: u8,
    pub check_blocker: u64,
    pub a_pins: [ u64; 64 ],

    pub node_info: NodeInfo,
    // repetition table
    // hash
    // other bb items
}

impl Default for State {
    fn default() -> Self {
        State { simple_board: [ EMPTY; 64 ],
                bit_board: BitBoard( [ 0; 14 ] ),
                to_move: 0,
                castling: 0,
                en_passant: NO_EP,
                halfmove_clock: 0,
                fullmove_count: 0,
                mg: MoveGen { ..Default::default() },
                attacked: 0,
                num_checks: 0,
                check_blocker: FULL_BOARD,
                a_pins: [ FULL_BOARD; 64 ],
                node_info: NodeInfo { ..Default::default() }, }
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
        if self.en_passant != NO_EP { output.push_str( &offset_to_algebraic( self.en_passant ) ) }
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
                        "-" => NO_EP,
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

        state.update_ad();
        state.node_info = NodeInfo { ..Default::default() };

        // FIXME: Implement a state check

        state
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
        if self.en_passant != NO_EP {
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

    pub fn ir_state( &self ) -> IRState {
        IRState{ castling: self.castling,
                 en_passant: self.en_passant,
                 halfmove_clock: self.halfmove_clock,
                 attacked: self.attacked,
                 num_checks: self.num_checks,
                 check_blocker: self.check_blocker,
                 a_pins: self.a_pins,
                 node_info: self.node_info.clone() }
    }

    pub fn set_ir_state( &mut self, irs: &IRState ) {
        self.castling = irs.castling;
        self.en_passant = irs.en_passant;
        self.halfmove_clock = irs.halfmove_clock;
        self.attacked = irs.attacked;
        self.num_checks = irs.num_checks;
        self.check_blocker = irs.check_blocker;
        self.a_pins = irs.a_pins;
        self.node_info = irs.node_info.clone();
    }

    pub fn make( &mut self, mv: &Move ) {
        let side = self.to_move;

        // Update simple_board and bit_board
        self.simple_board[ mv.from ] = EMPTY;
        self.simple_board[ mv.to ] = mv.piece;
        self.bit_board[ mv.piece ] ^= ( 1u64 << mv.from ) | ( 1u64 << mv.to );
        if mv.capture != EMPTY { self.bit_board[ mv.capture ] ^= 1u64 << mv.to; }

        // Update castling state and en_passant; handle promotion
        // Update simple_board and bit_board for Rook if castling
        let mut new_ep = NO_EP;
        match mv.piece {
            WHITE_PAWN => {
                if mv.is_promotion() {
                    self.simple_board[ mv.to ] = mv.promotion;
                    self.bit_board[ WHITE_PAWN ] ^= 1u64 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1u64 << mv.to;
                } else {
                    match mv.to - mv.from {
                        16 => { new_ep = mv.from + 8; }, // forward_2, set en_passant capture
                        _ => {
                            if self.en_passant == mv.to {
                                self.simple_board[ mv.to - 8 ] = EMPTY;
                                self.bit_board[ BLACK_PAWN ] ^= 1u64 << ( mv.to - 8 );
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
                    },
                    WQ_CASTLE => {
                        self.simple_board[ 0 ] = EMPTY;
                        self.simple_board[ 3 ] = WHITE_ROOK;
                        self.bit_board[ WHITE_ROOK ] ^= ROOK_WQC;
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
                    self.bit_board[ BLACK_PAWN ] ^= 1u64 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1u64 << mv.to;
                } else {
                    match mv.from - mv.to {
                        16 => { new_ep = mv.to + 8; }, // forward_2, set en_passant capture
                        _ => {
                            if self.en_passant == mv.to {
                                self.simple_board[ mv.to + 8 ] = EMPTY;
                                self.bit_board[ WHITE_PAWN ] ^= 1u64 << ( mv.to + 8 );
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
                    },
                    BQ_CASTLE => {
                        self.simple_board[ 56 ] = EMPTY;
                        self.simple_board[ 59 ] = BLACK_ROOK;
                        self.bit_board[ BLACK_ROOK ] ^= ROOK_BQC;
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

        self.bit_board.set_all(); // update 'ALL' bit_boards
        self.en_passant = new_ep; // set en_passant
        self.to_move ^= COLOR; // set side
        if side == BLACK { self.fullmove_count += 1; } // update fullmove_count
        if mv.piece == ( side | PAWN ) || mv.capture != EMPTY { self.halfmove_clock = 0; } else { self.halfmove_clock += 1; } // update halfmove_clock

        self.update_ad();
        self.node_info = NodeInfo { ..Default::default() };

        // update repetition table
        // update other blah
    }

    pub fn unmake( &mut self, mv: &Move, irs: &IRState ) {
        let side = self.to_move ^ COLOR; // side that just moved

        // Update simple_board and bit_board
        self.simple_board[ mv.from ] = mv.piece;
        self.simple_board[ mv.to ] = mv.capture;
        self.bit_board[ mv.piece ] ^= ( 1u64 << mv.from ) | ( 1u64 << mv.to );
        if mv.capture != EMPTY { self.bit_board[ mv.capture ] ^= 1u64 << mv.to; }

        // Undo castling and en_passant
        match mv.piece {
            WHITE_PAWN => {
                if mv.is_promotion() {
                    self.bit_board[ WHITE_PAWN ] ^= 1u64 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1u64 << mv.to;
                } else if irs.en_passant == mv.to {
                    self.simple_board[ mv.to - 8 ] = BLACK_PAWN;
                    self.bit_board[ BLACK_PAWN ] ^= 1u64 << ( mv.to - 8 );
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
                    self.bit_board[ BLACK_PAWN ] ^= 1u64 << mv.to;
                    self.bit_board[ mv.promotion ] ^= 1u64 << mv.to;
                } else if irs.en_passant == mv.to {
                    self.simple_board[ mv.to + 8 ] = WHITE_PAWN;
                    self.bit_board[ WHITE_PAWN ] ^= 1u64 << ( mv.to + 8 );
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

        // set IRState
        self.set_ir_state( irs );

        self.bit_board.set_all(); // update 'ALL' bit_boards
        self.to_move ^= COLOR; // set side
        if side == BLACK { self.fullmove_count -= 1; } // update fullmove_count

        // update repetition table
        // update other blah
    }

    pub fn ep_bb( &self ) -> u64 {
        if self.en_passant != NO_EP {
            1u64 << self.en_passant
        } else {
            0
        }
    }

    // All pseudo-legal moves for a given piece at a given pos, added to moves
    pub fn add_moves_from_bb( &self, piece: u8, pos: usize, moves_bb: &mut u64, moves: &mut Vec<Move> ) {
        let mut mv = Move { piece: piece, from: pos, to: ERR_POS, capture: EMPTY, promotion: EMPTY };
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

    pub fn update_ad( &mut self ) {
        // Updates the attack and defend information ---
        // Computes the following
        // attacked: all the squares attacked by enemies (including enemy pieces, which are 'defended')
        // num_checks
        // checks: all the squares that I can send a piece to, to block check
        // a_pins: absolute pins

        self.attacked = 0;
        self.num_checks = 0;
        self.check_blocker = FULL_BOARD;
        self.a_pins = [ FULL_BOARD; 64 ];

        let side = self.to_move;
        let opp_side = side ^ COLOR;
        let king = self.bit_board[ side | KING ];
        let king_pos = king.trailing_zeros() as usize;
        let friends = self.bit_board[ side | ALL ];
        let enemies = self.bit_board[ opp_side | ALL ];
        let occupancy = friends | enemies;

        // NOTE: We are computing attacks of enemy pieces.
        // So, the king, if in check is going to be blocking some sliding piece attacks, and we need to account for that.
        // This wouldn't come up when computing control of friendlies, because, if it's my move, then the enemy king can't be in check..
        let occupancy_wo_king = occupancy ^ king;

        let mut bb: u64;
        let mut pos: usize;
        let mut o_attacks: u64;
        let mut r_attacks: u64;
        let mut b_attacks: u64;

        /**** Checks and Attacked ****/

        // ROOK & QUEEN - orthogonal attacks
        bb = self.bit_board[ opp_side | ROOK ] | self.bit_board[ opp_side | QUEEN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            r_attacks = self.mg.r_moves( pos, occupancy_wo_king );
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
            self.attacked |= o_attacks;

            if o_attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= 1u64 << pos;
            }
        }

        // PAWN
        bb = self.bit_board[ opp_side | PAWN ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.p_captures( pos, opp_side );
            self.attacked |= o_attacks;

            if o_attacks & king != 0 {
                self.num_checks += 1;
                self.check_blocker &= 1u64 << pos;
            }
        }

        // KING
        bb = self.bit_board[ opp_side | KING ];
        while bb != 0 {
            pos = pop_lsb_pos( &mut bb );
            o_attacks = self.mg.k_captures( pos );
            self.attacked |= o_attacks;
        }

        /**** Pins ****/

        let mut vision: u64;
        let mut possibly_pinned: u64;
        let mut possibly_attacking: u64;
        let mut pinners: u64;
        let mut pinned_pos: usize;
        let mut pin: u64;

        // Diagonal pins
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

        // Orthogonal pins
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

        // ep pins - the special stuff
        if self.en_passant != NO_EP {
            let mut ep_killers = self.bit_board[ side | PAWN ] & self.mg.p_captures( self.en_passant, opp_side );
            if ep_killers != 0 {
                let ep_target: usize = match side {
                    WHITE => self.en_passant - 8,
                    BLACK => self.en_passant + 8,
                    _ => panic!( "Invalid color!" ),
                };

                // Everything except EP, basically if EP capture leads to check, ban it!
                pin = FULL_BOARD ^ ( 1u64 << self.en_passant );

                // Check if ep_target is diagonally pinned to our king
                // Btw, it cannot be pinned orthogonally (unless both ep_killer and ep_target are horizontally pinned to our king, which is handled after this)
                let mut ep_diag_pin = false;
                vision = self.mg.b_moves( king_pos, occupancy_wo_king );
                possibly_pinned = vision & ( 1u64 << ep_target );
                if possibly_pinned != 0 {
                    possibly_attacking = vision & enemies;
                    pinners = self.mg.b_moves( king_pos, occupancy_wo_king ^ possibly_pinned ) &
                              ( ( self.bit_board[ opp_side | QUEEN ] | self.bit_board[ opp_side | BISHOP ] ) & !possibly_attacking );
                    if pinners != 0 { ep_diag_pin = true; }
                }

                while ep_killers != 0 {
                    pinned_pos = pop_lsb_pos( &mut ep_killers );

                    if ep_diag_pin {
                        self.a_pins[ pinned_pos ] &= pin; // Everything except EP
                    } else {
                        // Check if both pawns are horizontally pinned to our king
                        let ep_clear: u64 = ( 1u64 << pinned_pos ) | ( 1u64 << ep_target );
                        vision = self.mg.r_moves( king_pos, occupancy_wo_king );
                        if vision & ep_clear != 0 {
                            possibly_attacking = vision & enemies;
                            pinners = self.mg.r_moves( king_pos, occupancy_wo_king ^ ep_clear ) &
                                      ( ( self.bit_board[ opp_side | QUEEN ] | self.bit_board[ opp_side | ROOK ] ) & !possibly_attacking );

                            if pinners != 0 {
                                self.a_pins[ pinned_pos ] &= pin;
                            }
                        }
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
                self.attacked & ( 1u64 << mv.to ) == 0 // King can't move into check
            } else if self.num_checks > 1 {
                false // Double check, only the King can move
            } else {
                ( self.check_blocker & self.a_pins[ mv.from ] ) & ( 1u64 << mv.to ) != 0 // The move shouldn't break out of an a_pin and should block check, if any
            }
        }
    }

    pub fn update_node_info( &mut self ) {
        if self.node_info.status == Status::Unknown {
            let moves = self.moves();

            // Hmm, this is faster than both (by ~25%) -
            // 1. legal_moves = moves.into_iter().filter( |x| self.is_legal( &x ) ).collect();
            // 2. legal_moves = moves.iter().filter( |x| self.is_legal( x ) ).map( |x| *x ).collect();
            let mut legal_moves: Vec<Move> = Vec::new();
            for mv in &moves {
                if self.is_legal( mv ) { legal_moves.push( *mv ); }
            }

            self.node_info.status = match ( legal_moves.len() == 0, self.num_checks > 0 ) {
                ( false, _ ) => Status::Ongoing,
                ( true, false ) => Status::Stalemate,
                ( true, true ) => Status::Checkmate,
            };

            self.node_info.legal_moves = legal_moves;
        }
    }

    pub fn legal_moves( &mut self ) -> Vec<Move> {
        self.update_node_info();
        self.node_info.legal_moves.clone()
    }

    pub fn perft( &mut self, depth: usize, debug: bool ) -> u64 {
        assert!( depth > 0, "Depth has to be greater than zero!" );
        
        let legal_moves = self.legal_moves();

        if depth == 1 {
            legal_moves.len() as u64
        } else {
            let mut nodes: u64 = 0;
            let mut temp: u64;
            let irs = self.ir_state();

            for mv in &legal_moves {
                self.make( mv );
                temp = self.perft( depth - 1, false );
                self.unmake( mv, &irs );

                if debug {
                    println!( "{}-{} = {}",
                        //"{}. {}{} {}-{} = {}",
                        //self.fullmove_count,
                        //if self.to_move == BLACK { ".. " } else { "" },
                        //piece_to_char( mv.piece ),
                        offset_to_algebraic( mv.from ),
                        offset_to_algebraic( mv.to ),
                        temp
                    );
                }

                nodes += temp;
            }

            nodes
        }
    }
}
