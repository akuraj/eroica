//! Defines required constants that will be used in this project

// Color types
pub const COLOR: u8 = 1;
pub const WHITE: u8 = 0;
pub const BLACK: u8 = COLOR;

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

// Castling types
pub const WK_CASTLE: u8 = 1;
pub const BK_CASTLE: u8 = WK_CASTLE << BLACK;
pub const WQ_CASTLE: u8 = 1 << 2;
pub const BQ_CASTLE: u8 = WQ_CASTLE << BLACK;

pub const W_CASTLE: u8 = WK_CASTLE | WQ_CASTLE;
pub const B_CASTLE: u8 = BK_CASTLE | BQ_CASTLE;

// BitBoard Constants
pub const FIRST_RANK: u64 = 0x00000000000000FFu64;
pub const FIRST_RANK_NE: u64 = 0x000000000000007Eu64; // NO EDGES
pub const A_FILE: u64 = 0x0101010101010101u64;
pub const A_FILE_NE: u64 = 0x0001010101010100u64; // NO EDGES
pub const A1_H8: u64 = 0x8040201008040201u64;
pub const A8_H1: u64 = 0x0102040810204080u64;
pub const LRT: u64 = 0x0080C0E0F0F8FCFE; // i > j
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
