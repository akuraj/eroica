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

pub const ROOK_MAGICS: [ u64; 64 ] = [ 36028970966466560u64, 18084767790534656u64, 5944786692669964672u64, 324267969331855490u64, 9871894850096333568u64, 4827867648174981376u64, 432369754624426120u64, 4683752548150083586u64,
                                       18155137071587456u64, 1153695629585678400u64, 4644887139983360u64, 72198881550534656u64, 4611826790283871232u64, 2432506765946914832u64, 9228438621133078529u64, 2594355050402940160u64,
                                       4670385095945764880u64, 4620693493633789952u64, 4755838590175543368u64, 38879830821048321u64, 144258674410457088u64, 1126449729896576u64, 9799837187775203472u64, 4629702615964582017u64,
                                       4647785255057498244u64, 9224568336646410240u64, 1747414249755377794u64, 875952363055087744u64, 216181580362679296u64, 2458967597715292288u64, 1152922606274347520u64, 4611726777669681220u64,
                                       108122950112116864u64, 5764678445862756864u64, 1157565979169918976u64, 589981447343837184u64, 306385529337808898u64, 140754676613632u64, 18023753015365633u64, 140944728916224u64,
                                       72198538759012354u64, 1801527812415307776u64, 2346375560613076994u64, 434597432761286784u64, 73183528304803968u64, 216174981204181120u64, 579277701160960256u64, 5765768899647897613u64,
                                       9922134688142080u64, 163290673015948416u64, 148621060278157440u64, 1161118928208384u64, 9259541605722095744u64, 2254000984686720u64, 722836546218165248u64, 585472367933851136u64,
                                       4521742139670786u64, 142946176476418u64, 27056818644599298u64, 1441719297496844545u64, 1738952543826151554u64, 4035788504176789634u64, 4647758850403930628u64, 576465189545902978u64 ];

pub const BISHOP_MAGICS: [ u64; 64 ] = [ 594617022275665984u64, 5485386553784021090u64, 164388035028327424u64, 289428025643631168u64, 299347946311688u64, 144264859242989569u64, 2450240290810765320u64, 72620683649359872u64,
                                         225779357006762048u64, 4740040841324725316u64, 306422904470380544u64, 5634052744806528u64, 432629247086952616u64, 144115772262731968u64, 304023825686912u64, 9367557739741601796u64,
                                         153122404666704128u64, 77688811714380808u64, 4614010729895428256u64, 2251821357269002u64, 145241122913386811u64, 4611827082601955584u64, 288305151550203904u64, 9583800886334390624u64,
                                         1196279933834368u64, 4794150408421648u64, 2254041804046853u64, 289356413564749952u64, 27167283256238082u64, 9225926202399686785u64, 1267264477990976u64, 1189760109405046784u64,
                                         572020993589762u64, 1127137126482435u64, 5066893346013441u64, 9234701439564120576u64, 594489445538071616u64, 4612249526735211008u64, 76845007321303043u64, 4612038141333767176u64,
                                         146684885801144320u64, 1154048263576487936u64, 4612249664517834752u64, 6755674592141824u64, 1196273113760001u64, 288547069864452224u64, 77899471702198793u64, 371731825732682048u64,
                                         1130307651635204u64, 1189253771147370569u64, 4612249108035404804u64, 19703250584601088u64, 4521262148240640u64, 2310366417364983936u64, 362858784672186496u64, 225322920044858369u64,
                                         2887094892033622080u64, 72629932806709377u64, 1152921547590140992u64, 9601674544070920320u64, 144116292421694466u64, 8075024541960110208u64, 6057361308544934976u64, 378303503216510080u64 ];

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

pub fn check_stored_magics( piece: u8 ) {
    let mask_fn: fn( u32 ) -> u64 = match piece {
        ROOK => rook_mask,
        BISHOP => bishop_mask,
        _ => panic!( "Invalid piece!" ),
    };

    let bits = match piece {
        ROOK => ROOK_BITS,
        BISHOP => BISHOP_BITS,
        _ => panic!( "Invalid piece!" ),
    };

    let attack: fn( u32, u64 ) -> u64 = match piece {
        ROOK => rook_attack,
        BISHOP => bishop_attack,
        _ => panic!( "Invalid piece!" ),
    };

    let magics = match piece {
        ROOK => ROOK_MAGICS,
        BISHOP => BISHOP_MAGICS,
        _ => panic!( "Invalid piece!" ),
    };

    for i in 0..64 {
        let pos: u32 = i as u32;
        let mask = mask_fn( pos );
        let num_bits = bits[ i ];

        assert!( num_bits == ( mask.count_ones() as u8 ) );
        let shift: u8 = 64 - num_bits;

        // Compute occupancies and attacks
        let occupancies = occupancies( mask );
        let attacks: Vec<u64> = occupancies.iter().map( |x| attack( pos, *x ) ).collect();

        let magic = magics[ i ];

        if !is_magic( magic, &occupancies, &attacks, shift ) {
            panic!( "Magic is not magical!\nPiece: {}, Pos: {}", if piece == ROOK { "Rook" } else { "Bishop" }, pos );
        }
    }
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
