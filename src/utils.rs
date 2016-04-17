//! Utils needed for various purposes

use consts::*;

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

pub fn pop_first( x: &mut u64 ) -> u32 {
    let probe = x.trailing_zeros();
    *x ^= 1 << probe;
    probe
}

// Compute block by expanding onto mask
pub fn expand_onto_mask( index: usize, n: u8, mask: u64 ) -> u64 {
    let mut block = mask;
    let mut temp = mask;
    let mut probe: u32;

    for i in 0..n {
        probe = pop_first( &mut temp );
        if ( index & ( 1 << i ) ) == 0 {
            block ^= 1 << probe;
        }
    }

    block
}

// Rook and Bishop attacks
pub fn rook_attack( pos: u32, block: u64 ) -> u64 {
    let ( i, j ) = file_rank( pos );

    let mut file_iter: u32;
    let mut rank_iter: u32;
    let mut pos_iter: u32;

    let mut attack: u64 = 0u64;

    // North
    rank_iter = j;
    pos_iter = pos;
    loop {
        if rank_iter == 7 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

        rank_iter += 1;
        pos_iter += 8;
        attack |= 1 << pos_iter;
    }

    // East
    file_iter = i;
    pos_iter = pos;
    loop {
        if file_iter == 7 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter += 1;
        pos_iter += 1;
        attack |= 1 << pos_iter;
    }

    // South
    rank_iter = j;
    pos_iter = pos;
    loop {
        if rank_iter == 0 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

        rank_iter -= 1;
        pos_iter -= 8;
        attack |= 1 << pos_iter;
    }

    // South
    file_iter = i;
    pos_iter = pos;
    loop {
        if file_iter == 0 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter -= 1;
        pos_iter -= 1;
        attack |= 1 << pos_iter;
    }

    attack
}

pub fn bishop_attack( pos: u32, block: u64 ) -> u64 {
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
        if file_iter == 7 || rank_iter == 7 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

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
        if file_iter == 7 || rank_iter == 0 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

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
        if file_iter == 0 || rank_iter == 0 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

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
        if file_iter == 0 || rank_iter == 7 || ( block & ( 1 << pos_iter ) ) != 0 { break; }

        file_iter -= 1;
        rank_iter += 1;
        pos_iter += 7;
        attack |= 1 << pos_iter;
    }

    attack
}
