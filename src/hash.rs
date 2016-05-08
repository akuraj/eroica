//! An implementation of Zobrist Hash - provides an interface to return hash by piece/pos

use std::default::Default;
use rand::{ Rng, SeedableRng, ChaChaRng };

pub struct HashGen {
    pub side_hash: u64, // ON when White to move
    pub piece_hash: [ u64; 768 ],
    pub castling_hash: [ u64; 16 ], // One for each state (2^4)
    pub ep_hash: [ u64; 8 ],  // One for each file - only used if ep_possible
}

impl Default for HashGen {
    fn default() -> Self {
        let mut hg = HashGen { side_hash: 0,
                               piece_hash: [ 0; 768 ],
                               castling_hash: [ 0; 16 ],
                               ep_hash: [ 0; 8 ], };

        hg.init();

        hg
    }
}

impl HashGen {
    pub fn init( &mut self ) {
        let seed: &[ _ ] = &[ 9, 11, 19, 36 ]; // 9/11/1936 is Tal's birthdate
        let mut rng: ChaChaRng = SeedableRng::from_seed( seed );

        // Side
        self.side_hash = rng.gen::<u64>();

        // Pieces
        for v in self.piece_hash.iter_mut() {
            *v = rng.gen::<u64>();
        }

        // Castling
        for v in self.castling_hash.iter_mut() {
            *v = rng.gen::<u64>();
        }

        // en_passant
        for v in self.ep_hash.iter_mut() {
            *v = rng.gen::<u64>();
        }
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