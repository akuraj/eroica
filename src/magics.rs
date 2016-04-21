//! Magics

use consts::*;
use utils::*;
use rand::{ Rng, thread_rng };

// Hash size in bits for each square - separate for Rooks and Bishops
// Equal to the number of bits set in the mask
pub const ROOK_BITS: [ u8; 64 ] = [ 12, 11, 11, 11, 11, 11, 11, 12,
                                    11, 10, 10, 10, 10, 10, 10, 11,
                                    11, 10, 10, 10, 10, 10, 10, 11,
                                    11, 10, 10, 10, 10, 10, 10, 11,
                                    11, 10, 10, 10, 10, 10, 10, 11,
                                    11, 10, 10, 10, 10, 10, 10, 11,
                                    11, 10, 10, 10, 10, 10, 10, 11,
                                    12, 11, 11, 11, 11, 11, 11, 12 ];

pub const BISHOP_BITS: [ u8; 64 ] = [ 6, 5, 5, 5, 5, 5, 5, 6,
                                      5, 5, 5, 5, 5, 5, 5, 5,
                                      5, 5, 7, 7, 7, 7, 5, 5,
                                      5, 5, 7, 9, 9, 7, 5, 5,
                                      5, 5, 7, 9, 9, 7, 5, 5,
                                      5, 5, 7, 7, 7, 7, 5, 5,
                                      5, 5, 5, 5, 5, 5, 5, 5,
                                      6, 5, 5, 5, 5, 5, 5, 6 ];

pub fn magic_hash( magic: u64, occupancy: u64, shift: u8 ) -> usize {
    ( magic.wrapping_mul( occupancy ) >> shift ) as usize
}

pub fn is_magic( guess: u64, occupancies: &[ u64 ], attacks: &[ u64 ], shift: u8 ) -> bool {
    let mut used: Vec<u64> = vec![ 0u64; occupancies.len() ];
    let mut hash: usize;

    for ( i, occupancy ) in occupancies.iter().enumerate() {
        hash = magic_hash( guess, *occupancy, shift );
        if used[ hash ] == 0u64 {
            used[ hash ] = attacks[ i ];
        } else if used[ hash ] != attacks[ i ] {
            return false;
        }
    }

    true
}

pub fn magic( pos: u32, piece: u8, verbose: bool ) -> u64 {
    assert!( pos < 64, "Square address out of bounds!" );

    let mask = match piece {
        ROOK => rook_mask( pos ),
        BISHOP => bishop_mask( pos ),
        _ => panic!( "Invalid piece!" ),
    };

    let num_bits = match piece {
        ROOK => ROOK_BITS[ pos as usize ],
        BISHOP => BISHOP_BITS[ pos as usize ],
        _ => panic!( "Invalid piece!" ),
    };

    let attack: fn( u32, u64 ) -> u64 = match piece {
        ROOK => rook_attack,
        BISHOP => bishop_attack,
        _ => panic!( "Invalid piece!" ),
    };

    assert!( num_bits == ( mask.count_ones() as u8 ) );
    let shift: u8 = 64 - num_bits;

    // Compute occupancies and attacks
    let occupancies = occupancies( mask );
    let attacks: Vec<u64> = occupancies.iter().map( |x| attack( pos, *x ) ).collect();

    // Trial and error to find the magic
    let mut rng = thread_rng();
    let mut guess: u64;
    let mut tries = 0;

    'main: loop {
        'guess: loop {
            guess = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>(); // num_bits: Mean = 8, StdDev = 2.65
            if magic_hash( guess, mask, 56 ).count_ones() >= 6 { break 'guess; }
        }

        tries += 1;

        if is_magic( guess, &occupancies, &attacks, shift ) {
            if verbose {
                println!( "pos: {}, piece: {}, tries: {}\nmagic: {}", pos, if piece == ROOK { "Rook" } else { "Bishop" }, tries, guess );
            }

            return guess;
        }
    }
}
