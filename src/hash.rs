//! An implementation of Zobrist Hash - provides an interface to return hash by piece/pos

use rand::{ Rng, SeedableRng };
use rand_chacha::ChaChaRng;

pub struct HashGen {
    pub side_hash: u64, // ON when White to move
    pub piece_hash: [ u64; 768 ],
    pub castling_hash: [ u64; 16 ], // One for each state (2^4)
    pub ep_hash: [ u64; 8 ],  // One for each file - only used if ep_possible
}

impl Default for HashGen {
    fn default() -> Self {
        Self::new()
    }
}

impl HashGen {
    pub fn new() -> Self {
        // Seed: 9/11/1936 is Tal's birthdate
        let mut rng: ChaChaRng = SeedableRng::from_seed( [ 9, 11, 19, 36,
                                                           9, 11, 19, 36,
                                                           9, 11, 19, 36,
                                                           9, 11, 19, 36,
                                                           9, 11, 19, 36,
                                                           9, 11, 19, 36,
                                                           9, 11, 19, 36,
                                                           9, 11, 19, 36 ] );

        let mut hg = HashGen { side_hash: 0,
                               piece_hash: [ 0; 768 ],
                               castling_hash: [ 0; 16 ],
                               ep_hash: [ 0; 8 ], };

        // Side
        hg.side_hash = rng.gen::<u64>();

        // Pieces
        for v in hg.piece_hash.iter_mut() {
            *v = rng.gen::<u64>();
        }

        // Castling
        for v in hg.castling_hash.iter_mut() {
            *v = rng.gen::<u64>();
        }

        // en_passant
        for v in hg.ep_hash.iter_mut() {
            *v = rng.gen::<u64>();
        }

        hg
    }

    pub fn piece( &self, piece: u8, pos: usize ) -> u64 {
        self.piece_hash[ ( piece as usize ) * 64 + pos ]
    }

    pub fn castling( &self, castling: u8 ) -> u64 {
        self.castling_hash[ castling as usize ]
    }

    pub fn ep( &self, pos: usize ) -> u64 {
        self.ep_hash[ pos % 8 ]
    }
}
