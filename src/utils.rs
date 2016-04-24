//! Utils needed for various purposes

use consts::*;
use std::u64::MAX;
use std::cmp;

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
pub fn file_rank( pos: usize ) -> ( usize, usize ) {
    ( pos % 8, pos / 8 )
}

// char to piece
pub fn char_to_piece( c: char ) -> u8 {
    match c {
        'P' => WHITE_PAWN,
        'N' => WHITE_KNIGHT,
        'B' => WHITE_BISHOP,
        'R' => WHITE_ROOK,
        'Q' => WHITE_QUEEN,
        'K' => WHITE_KING,
        'p' => BLACK_PAWN,
        'n' => BLACK_KNIGHT,
        'b' => BLACK_BISHOP,
        'r' => BLACK_ROOK,
        'q' => BLACK_QUEEN,
        'k' => BLACK_KING,
         _  => panic!( "Invalid piece: {}", c ),
    }
}

// char to piece
pub fn piece_to_char( piece: u8 ) -> char {
    match piece {
        WHITE_PAWN => 'P',
        WHITE_KNIGHT => 'N',
        WHITE_BISHOP => 'B',
        WHITE_ROOK => 'R',
        WHITE_QUEEN => 'Q',
        WHITE_KING => 'K',
        BLACK_PAWN => 'p',
        BLACK_KNIGHT => 'n',
        BLACK_BISHOP => 'b',
        BLACK_ROOK => 'r',
        BLACK_QUEEN => 'q',
        BLACK_KING => 'k',
        EMPTY => ' ',
         _  => panic!( "Invalid piece: {}", piece ),
    }
}

pub fn algebraic_to_offset( square: &str ) -> usize {
    assert!( square.chars().count() == 2, "Invalid algebraic address \"{}\"", square );
    let mut char_iter = square.chars().enumerate();
    let mut offset: usize = 0;

    while let Some( ( i, c ) ) = char_iter.next() {
        match i {
            0 => {
                assert!( 'a' <= c && c <= 'h', "Invalid file: {}", c );
                offset += c as usize - 'a' as usize;
            },
            1 => {
                if let Some( rank_number ) = c.to_digit( 10 ) {
                    assert!( 1 <= rank_number && rank_number <= 8, "Invalid rank: {}", c );
                    offset += ( rank_number as usize - 1 ) * 8;
                } else {
                    panic!( "Invalid rank: {}", c );
                }
            },
            _ => panic!( "Invalid algebraic address \"{}\"", square ),
        }
    }

    offset
}

pub fn offset_to_algebraic( offset: usize ) -> String {
    assert!( offset < 64, "Square address out of bounds!" );

    let mut algebraic = String::new();
    let ( i, j ) = file_rank( offset );
    let file = match i {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => panic!( "Invalid file: {}!", i ),
    };

    algebraic.push( file );
    algebraic.push_str( &( ( j + 1 ).to_string() ) );

    algebraic
}

// Rook and Bishop Masks
pub fn rook_mask( pos: usize ) -> u64 {
    let ( i, j ) = file_rank( pos );
    let not_rook_bb: u64 = !( 1u64 << pos );
    let rook_mask: u64 = ( A_FILE_NE << i ) ^ ( FIRST_RANK_NE << ( 8 * j ) );

    rook_mask & not_rook_bb
}

pub fn bishop_mask( pos: usize ) -> u64 {
    let ( i, j ) = file_rank( pos );
    let s = i + j;

    let mut bishop_mask: u64;

    if i > j {
        bishop_mask = A1_H8 >> ( 8 * ( i - j ) );
    } else {
        bishop_mask = A1_H8 << ( 8 * ( j - i ) );
    }

    if s > 7 {
        bishop_mask ^= A8_H1 << ( 8 * s - 56 );
    } else {
        bishop_mask ^= A8_H1 >> ( 56 - 8 * s );
    }

    bishop_mask & NOT_EDGES
}

pub fn occupancies( mask: u64 ) -> Vec<u64> {
    // Carry-Rippler implementation
    let mut occupancy: u64 = 0;
    let mut occupancies: Vec<u64> = Vec::new();

    loop {
        occupancies.push( occupancy );
        occupancy = occupancy.wrapping_sub( mask ) & mask;
        if occupancy == 0 { break; }
    }

    occupancies
}

// Rook and Bishop attacks
pub fn rook_attack( pos: usize, occupancy: u64 ) -> u64 {
    let ( i, j ) = file_rank( pos );

    let mut file_iter: usize;
    let mut rank_iter: usize;
    let mut pos_iter: usize;

    let mut attack: u64 = 0u64;

    // North
    rank_iter = j;
    pos_iter = pos;
    loop {
        if rank_iter == 7 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        rank_iter += 1;
        pos_iter += 8;
        attack |= 1 << pos_iter;
    }

    // East
    file_iter = i;
    pos_iter = pos;
    loop {
        if file_iter == 7 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter += 1;
        pos_iter += 1;
        attack |= 1 << pos_iter;
    }

    // South
    rank_iter = j;
    pos_iter = pos;
    loop {
        if rank_iter == 0 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        rank_iter -= 1;
        pos_iter -= 8;
        attack |= 1 << pos_iter;
    }

    // South
    file_iter = i;
    pos_iter = pos;
    loop {
        if file_iter == 0 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter -= 1;
        pos_iter -= 1;
        attack |= 1 << pos_iter;
    }

    attack
}

pub fn bishop_attack( pos: usize, occupancy: u64 ) -> u64 {
    let ( i, j ) = file_rank( pos );

    let mut file_iter: usize;
    let mut rank_iter: usize;
    let mut pos_iter: usize;

    let mut attack: u64 = 0u64;

    // NE
    file_iter = i;
    rank_iter = j;
    pos_iter = pos;
    loop {
        if file_iter == 7 || rank_iter == 7 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter += 1;
        rank_iter += 1;
        pos_iter += 9;
        attack |= 1 << pos_iter;
    }

    // SE
    file_iter = i;
    rank_iter = j;
    pos_iter = pos;
    loop {
        if file_iter == 7 || rank_iter == 0 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter += 1;
        rank_iter -= 1;
        pos_iter -= 7;
        attack |= 1 << pos_iter;
    }

    // SW
    file_iter = i;
    rank_iter = j;
    pos_iter = pos;
    loop {
        if file_iter == 0 || rank_iter == 0 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter -= 1;
        rank_iter -= 1;
        pos_iter -= 9;
        attack |= 1 << pos_iter;
    }

    // NW
    file_iter = i;
    rank_iter = j;
    pos_iter = pos;
    loop {
        if file_iter == 0 || rank_iter == 7 || ( occupancy & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter -= 1;
        rank_iter += 1;
        pos_iter += 7;
        attack |= 1 << pos_iter;
    }

    attack
}

// Returns the rectangle of influence
pub fn influence( pos: usize, size: usize ) -> u64 {
    let ( i, j ) = file_rank( pos );

    let influence_rank: u64 = match ( ( j + size ) < 7, j > size ) {
        ( true, true ) => ( MAX >> ( 56 - 8 * ( j + size ) ) ) & ( MAX << ( 8 * ( j - size ) ) ),
        ( true, false ) => MAX >> ( 56 - 8 * ( j + size ) ),
        ( false, true ) => MAX << ( 8 * ( j - size ) ),
        ( false, false ) => panic!( "You're trying to influence too much - just stop!" ),
    };

    let mut influence_file: u64 = 0u64;
    let mut iter = cmp::max( i, size ) - size;
    let mut file = A_FILE << iter;
    let end = cmp::min( i + size, 7 );

    loop {
        influence_file |= file;
        file <<= 1;
        if iter == end { break; } else { iter += 1; }
    }

    influence_rank & influence_file
}

pub fn knight_attack( pos: usize ) -> u64 {
    let attack: u64 = if pos > 18 {
        KNIGHT_PATTERN_C3 << ( pos - 18 )
    } else {
        KNIGHT_PATTERN_C3 >> ( 18 - pos )
    };

    attack & influence( pos, 2 )
}

pub fn king_attack( pos: usize ) -> u64 {
    ( 1u64 << pos ) ^ influence( pos, 1 )
}

pub fn pawn_capture( pos: usize, color: u8 ) -> u64 {
    let ( _i, j ) = file_rank( pos );

    if j == 0 || j == 7 {
        panic!( "A pawn cannot be on the first or the eigth rank!" );
    }

    let ( capture_rank, attack ): ( usize, u64 ) = match color {
        WHITE => ( j + 1, PCP_W_A2 << ( pos - 8 ) ),
        BLACK => ( j - 1, PCP_B_H7 >> ( 55 - pos ) ),
        _ => panic!( "Invalid color!" ),
    };

    let capture_rank_fill = FIRST_RANK << ( 8 * capture_rank );

    attack & capture_rank_fill
}

pub fn pawn_forward( pos: usize, color: u8, occupancy: u64 ) -> u64 {
    let ( _i, j ) = file_rank( pos );

    if j == 0 || j == 7 {
        panic!( "A pawn cannot be on the first or the eigth rank!" );
    }

    let forward_1: u64 = match color {
        WHITE => 1u64 << ( pos + 8 ),
        BLACK => 1u64 << ( pos - 8 ),
        _ => panic!( "Invalid color!" ),
    };

    if ( forward_1 & occupancy ) != 0 {
        0u64
    } else {
        if ( j == 1 && color == WHITE ) || ( j == 6 && color == BLACK )
        {
            // Pawn hasn't moved yet
            let forward_2: u64 = match color {
                WHITE => 1u64 << ( pos + 16 ),
                BLACK => 1u64 << ( pos - 16 ),
                _ => panic!( "Invalid color!" ),
            };

            forward_1 | ( forward_2 ^ ( occupancy & forward_2 ) )
        } else {
            forward_1
        }
    }
}
