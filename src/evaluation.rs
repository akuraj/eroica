//! Static (Heuristic) Evaluation

/* Values in centi-pawns */

// FIXME: Currently, the piece values are borrowed from Stockfish. Will tune this at some point in the future.

// Piece Values
pub const PAWN_VALUE_MG: i32 = 80;
pub const KNIGHT_VALUE_MG: i32 = 320;
pub const BISHOP_VALUE_MG: i32 = 330;
pub const ROOK_VALUE_MG: i32 = 500;
pub const QUEEN_VALUE_MG: i32 = 990;

pub const PAWN_VALUE_EG: i32 = 100;
pub const KNIGHT_VALUE_EG: i32 = 350;
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

// Square index map for PSTs
pub const SQUARE_MAP: [ usize; 64 ] = [ 56, 57, 58, 59, 60, 61, 62, 63,
                                        48, 49, 50, 51, 52, 53, 54, 55,
                                        40, 41, 42, 43, 44, 45, 46, 47,
                                        32, 33, 34, 35, 36, 37, 38, 39,
                                        24, 25, 26, 27, 28, 29, 30, 31,
                                        16, 17, 18, 19, 20, 21, 22, 23,
                                         8,  9, 10, 11, 12, 13, 14, 15,
                                         0,  1,  2,  3,  4,  5,  6,  7 ];

 // Piece square tables
 // https://chessprogramming.wikispaces.com/Simplified+evaluation+function
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
                                       0,  0,  5,  5,  5,  5,  0, -5,
                                     -10,  5,  5,  5,  5,  5,  0,-10,
                                     -10,  0,  5,  0,  0,  0,  0,-10,
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
