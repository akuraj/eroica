//! Defines required constants that will be used in this project

// Start FEN
pub const START_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Color types
pub const COLOR: u8 = 1;
pub const WHITE: u8 = 0;
pub const BLACK: u8 = COLOR;

pub const COLOR_MASK: u8 = !COLOR;

// Piece types ( including EMPTY and ALL )
pub const EMPTY: u8 = !0;
pub const PAWN: u8 = 0;
pub const KNIGHT: u8 = 1 << 1;
pub const BISHOP: u8 = 2 << 1;
pub const ROOK: u8 = 3 << 1;
pub const QUEEN: u8 = 4 << 1;
pub const KING: u8 = 5 << 1;
pub const ALL: u8 = 6 << 1;

// Enumerate pieces
pub const WHITE_PAWN: u8 = WHITE | PAWN;
pub const WHITE_KNIGHT: u8 = WHITE | KNIGHT;
pub const WHITE_BISHOP: u8 = WHITE | BISHOP;
pub const WHITE_ROOK: u8 = WHITE | ROOK;
pub const WHITE_QUEEN: u8 = WHITE | QUEEN;
pub const WHITE_KING: u8 = WHITE | KING;
pub const WHITE_ALL: u8 = WHITE | ALL;
pub const BLACK_PAWN: u8 = BLACK | PAWN;
pub const BLACK_KNIGHT: u8 = BLACK | KNIGHT;
pub const BLACK_BISHOP: u8 = BLACK | BISHOP;
pub const BLACK_ROOK: u8 = BLACK | ROOK;
pub const BLACK_QUEEN: u8 = BLACK | QUEEN;
pub const BLACK_KING: u8 = BLACK | KING;
pub const BLACK_ALL: u8 = BLACK | ALL;

// All types except kings
pub const ALL_PIECE_TYPES: [ u8; 10 ] = [ WHITE_PAWN, WHITE_KNIGHT, WHITE_BISHOP, WHITE_ROOK, WHITE_QUEEN,
                                          BLACK_PAWN, BLACK_KNIGHT, BLACK_BISHOP, BLACK_ROOK, BLACK_QUEEN ];

// Castling types
pub const WK_CASTLE: u8 = 1;
pub const BK_CASTLE: u8 = WK_CASTLE << BLACK;
pub const WQ_CASTLE: u8 = 1 << 2;
pub const BQ_CASTLE: u8 = WQ_CASTLE << BLACK;

pub const W_CASTLE: u8 = WK_CASTLE | WQ_CASTLE;
pub const B_CASTLE: u8 = BK_CASTLE | BQ_CASTLE;

// Other
pub const ERR_POS: usize = !0usize; // Error Position
pub const FULL_BOARD: u64 = !0u64; // All bits set on the board

// BitBoard Constants
pub const FIRST_RANK: u64 = 0x00000000000000FFu64;
pub const FIRST_RANK_NE: u64 = 0x000000000000007Eu64; // NO EDGES
pub const A_FILE: u64 = 0x0101010101010101u64;
pub const A_FILE_NE: u64 = 0x0001010101010100u64; // NO EDGES
pub const A1_H8: u64 = 0x8040201008040201u64;
pub const A8_H1: u64 = 0x0102040810204080u64;
pub const LRT: u64 = 0x0080C0E0F0F8FCFEu64; // i > j
pub const ULT: u64 = !LRT; // i <= j // includes A1_H8
pub const URT: u64 = 0xFEFCF8F0E0C08000u64; // i + j > 7
pub const LLT: u64 = !URT; // i + j <= 7 // includes A8_H1
pub const EDGES: u64 = 0xFF818181818181FFu64;
pub const NOT_EDGES: u64 = !EDGES;
pub const KNIGHT_PATTERN_C3: u64 = 0x0000000A1100110Au64;
pub const PCP_W_A2: u64 = 0x0000000000028000u64; // Pawn Capture Pattern
pub const PCP_B_H7: u64 = 0x0001400000000000u64; // Pawn Capture Pattern
pub const ROOK_WKC: u64 = 0x00000000000000A0u64; // Castling Pattern
pub const ROOK_WQC: u64 = 0x0000000000000009u64; // Castling Pattern
pub const ROOK_BKC: u64 = ROOK_WKC << 56; // Castling Pattern
pub const ROOK_BQC: u64 = ROOK_WQC << 56; // Castling Pattern
pub const WK_CASTLE_BB: u64 = 0x0000000000000040u64;
pub const WQ_CASTLE_BB: u64 = 0x0000000000000004u64;
pub const BK_CASTLE_BB: u64 = WK_CASTLE_BB << 56;
pub const BQ_CASTLE_BB: u64 = WQ_CASTLE_BB << 56;
pub const WKCR: u64 = 0x0000000000000070u64;
pub const WKCR_OCC: u64 = 0x0000000000000060u64;
pub const WQCR: u64 = 0x000000000000001Cu64;
pub const WQCR_OCC: u64 = 0x000000000000000Eu64;
pub const BKCR: u64 = WKCR << 56;
pub const BKCR_OCC: u64 = WKCR_OCC << 56;
pub const BQCR: u64 = WQCR << 56;
pub const BQCR_OCC: u64 = WQCR_OCC << 56;
pub const ALL_WHITE_SQUARES: u64 = 0x55AA55AA55AA55AAu64;
pub const ALL_BLACK_SQUARES: u64 = 0xAA55AA55AA55AA55u64;

// Position Constants
pub const WK_START: usize = 4;
pub const WK_KS_CASTLE: usize = 6;
pub const WK_QS_CASTLE: usize = 2;
pub const BK_START: usize = 60;
pub const BK_KS_CASTLE: usize = 62;
pub const BK_QS_CASTLE: usize = 58;

/*****************************
**** Evaluation Constants ****
*****************************/

/* Values in centi-pawns */

pub const INF_VALUE: i32 = 1 << 20;

// Piece Values
// FIXME: Currently, the piece values are borrowed from Stockfish. Will tune this at some point in the future.
pub const PAWN_VALUE_MG: i32 = 80;
pub const KNIGHT_VALUE_MG: i32 = 320;
pub const BISHOP_VALUE_MG: i32 = 330;
pub const ROOK_VALUE_MG: i32 = 500;
pub const QUEEN_VALUE_MG: i32 = 990;

pub const PAWN_VALUE_EG: i32 = 100;
pub const KNIGHT_VALUE_EG: i32 = 345;
pub const BISHOP_VALUE_EG: i32 = 355;
pub const ROOK_VALUE_EG: i32 = 530;
pub const QUEEN_VALUE_EG: i32 = 1040;

// Game Phase Non Pawn Material (NPM) Limits
// MG values of the pieces are used to compute NPM
// Don't extrapolate outisde the NPM Limits. FIXME: Revisit this later.
// Use a Multiplier to preserve resolution -> MG_PHASE
pub const MG_NPM_LIMIT: i32 = 6130;
pub const EG_NPM_LIMIT: i32 = 1570;
pub const MG_PHASE: i32 = 128;

// Game Termination Values
pub const DRAW_VALUE: i32 = 0;
pub const MATE_VALUE: i32 = 32000;

// Tempo Bonus
// Will depend on your evaluation function of course. The PST Evaluation doesn't account for Tempo at all.
// Appropriate for "quiet" positions.
// FIXME: Currently using constant value of Tempo throughout. Revisit this later. Literally just guessed a number....
pub const TEMPO_BONUS: i32 = 16;

// Bishop Pair Bonus
pub const BISHOP_PAIR_BONUS: i32 = 50;

// Piece square tables
// https://chessprogramming.wikispaces.com/Simplified+evaluation+function
// NOTE: The columns are inverted for both White [ h -> a ] and Black [ a -> h ]. KEEP THE PST SYMMETRIC AROUND THE VERTICAL AXIS!
pub const PAWN_PST: [ i32; 64 ] = [  0,  0,  0,  0,  0,  0,  0,  0,
                                    50, 50, 50, 50, 50, 50, 50, 50,
                                    10, 10, 20, 30, 30, 20, 10, 10,
                                     5,  5, 10, 25, 25, 10,  5,  5,
                                     0,  0,  0, 20, 20,  0,  0,  0,
                                     5, -5,-10,  0,  0,-10, -5,  5,
                                     5, 10, 10,-20,-20, 10, 10,  5,
                                     0,  0,  0,  0,  0,  0,  0,  0 ];

pub const KNIGHT_PST: [ i32; 64 ] = [ -50,-40,-30,-30,-30,-30,-40,-50,
                                      -40,-20,  0,  0,  0,  0,-20,-40,
                                      -30,  0, 10, 15, 15, 10,  0,-30,
                                      -30,  5, 15, 20, 20, 15,  5,-30,
                                      -30,  0, 15, 20, 20, 15,  0,-30,
                                      -30,  5, 10, 15, 15, 10,  5,-30,
                                      -40,-20,  0,  5,  5,  0,-20,-40,
                                      -50,-40,-30,-30,-30,-30,-40,-50 ];

pub const BISHOP_PST: [ i32; 64 ] = [ -20,-10,-10,-10,-10,-10,-10,-20,
                                      -10,  0,  0,  0,  0,  0,  0,-10,
                                      -10,  0,  5, 10, 10,  5,  0,-10,
                                      -10,  5,  5, 10, 10,  5,  5,-10,
                                      -10,  0, 10, 10, 10, 10,  0,-10,
                                      -10, 10, 10, 10, 10, 10, 10,-10,
                                      -10,  5,  0,  0,  0,  0,  5,-10,
                                      -20,-10,-10,-10,-10,-10,-10,-20 ];

pub const ROOK_PST: [ i32; 64 ] = [ 0,  0,  0,  0,  0,  0,  0,  0,
                                    5, 10, 10, 10, 10, 10, 10,  5,
                                   -5,  0,  0,  0,  0,  0,  0, -5,
                                   -5,  0,  0,  0,  0,  0,  0, -5,
                                   -5,  0,  0,  0,  0,  0,  0, -5,
                                   -5,  0,  0,  0,  0,  0,  0, -5,
                                   -5,  0,  0,  0,  0,  0,  0, -5,
                                    0,  0,  0,  5,  5,  0,  0,  0 ];

pub const QUEEN_PST: [ i32; 64 ] = [ -20,-10,-10, -5, -5,-10,-10,-20,
                                     -10,  0,  0,  0,  0,  0,  0,-10,
                                     -10,  0,  5,  5,  5,  5,  0,-10,
                                      -5,  0,  5,  5,  5,  5,  0, -5,
                                      -5,  0,  5,  5,  5,  5,  0, -5,
                                     -10,  0,  5,  5,  5,  5,  0,-10,
                                     -10,  0,  0,  0,  0,  0,  0,-10,
                                     -20,-10,-10, -5, -5,-10,-10,-20 ];

pub const KING_MG_PST: [ i32; 64 ] = [ -30,-40,-40,-50,-50,-40,-40,-30,
                                       -30,-40,-40,-50,-50,-40,-40,-30,
                                       -30,-40,-40,-50,-50,-40,-40,-30,
                                       -30,-40,-40,-50,-50,-40,-40,-30,
                                       -20,-30,-30,-40,-40,-30,-30,-20,
                                       -10,-20,-20,-20,-20,-20,-20,-10,
                                        20, 20,  0,  0,  0,  0, 20, 20,
                                        20, 30, 10,  0,  0, 10, 30, 20 ];

pub const KING_EG_PST: [ i32; 64 ] = [ -50,-40,-30,-20,-20,-30,-40,-50,
                                       -30,-20,-10,  0,  0,-10,-20,-30,
                                       -30,-10, 20, 30, 30, 20,-10,-30,
                                       -30,-10, 30, 40, 40, 30,-10,-30,
                                       -30,-10, 30, 40, 40, 30,-10,-30,
                                       -30,-10, 20, 30, 30, 20,-10,-30,
                                       -30,-30,  0,  0,  0,  0,-30,-30,
                                       -50,-30,-30,-30,-30,-30,-30,-50 ];
