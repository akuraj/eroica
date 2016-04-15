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

// Castling types
pub const WK_CASTLE: u8 = 1;
pub const BK_CASTLE: u8 = WK_CASTLE << BLACK;
pub const WQ_CASTLE: u8 = 1 << 2;
pub const BQ_CASTLE: u8 = WQ_CASTLE << BLACK;

pub const W_CASTLE: u8 = WK_CASTLE | WQ_CASTLE;
pub const B_CASTLE: u8 = BK_CASTLE | BQ_CASTLE;

// BitBoard Constants
pub const FIRST_ROW: u64 = 0x00000000000000FFu64;
pub const A_FILE: u64 = 0x0101010101010101u64;
pub const A1_H8: u64 = 0x8040201008040201u64;
pub const A8_H1: u64 = 0x0102040810204080u64;
pub const LRT: u64 = 0x0080C0E0F0F8FCFE; // i > j
pub const ULT: u64 = !LRT; // i <= j // includes A1_H8
pub const EDGES: u64 = 0xFF818181818181FFu64;
pub const NOT_EDGES: u64 = !EDGES;
