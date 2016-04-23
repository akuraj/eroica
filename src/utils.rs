//! Utils needed for various purposes

use consts::*;
use std::u64::MAX;

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
    let rook_bb: u64 = 1 << pos;
    let rook_mask: u64 = ( A_FILE_NE << i ) ^ ( FIRST_RANK_NE << ( 8 * j ) );

    rook_mask & ( rook_mask ^ rook_bb )
}

pub fn bishop_mask( pos: u32 ) -> u64 {
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
pub fn rook_attack( pos: u32, occupancy: u64 ) -> u64 {
    let ( i, j ) = file_rank( pos );

    let mut file_iter: u32;
    let mut rank_iter: u32;
    let mut pos_iter: u32;

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

pub fn bishop_attack( pos: u32, occupancy: u64 ) -> u64 {
    let ( i, j ) = file_rank( pos );

    let mut file_iter: u32;
    let mut rank_iter: u32;
    let mut pos_iter: u32;

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
pub fn knight_influence( pos: u32 ) -> u64 {
    let ( i, j ) = file_rank( pos );

    let influence_rank: u64 = match ( j < 5, j > 2 ) {
        ( true, true ) => ( MAX >> ( 40 - 8 * j ) ) & ( MAX << ( 8 * j - 16 ) ),
        ( true, false ) => MAX >> ( 40 - 8 * j ),
        ( false, true ) => MAX << ( 8 * j - 16 ),
        ( false, false ) => panic!( "How?! What sorcery is this!?" ),
    };

    let mut influence_file: u64 = 0u64;
    let mut file = A_FILE;
    for index in 0..8 {
        if ( ( index + 3 ) > i ) && ( index < ( i + 3 ) ) {
            influence_file |= file;
        }

        file <<= 1;
    }

    influence_rank & influence_file
}

pub fn knight_attack( pos: u32 ) -> u64 {
    let attack: u64 = if pos > 18 {
        KNIGHT_PATTERN_C3 << ( pos - 18 )
    } else {
        KNIGHT_PATTERN_C3 >> ( 18 - pos )
    };

    attack & knight_influence( pos )
}
