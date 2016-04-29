//! Attacks - defines an interface that provides functions to get the possible attacks/moves of pieces
//! Stores attacks for non-sliding pieces, uses magics to compute attacks for sliding pieces

use std::default::Default;
use consts::*;
use utils::*;
use magics::*;

pub struct Attacks {
    pub white_pawn_attacks: [ u64; 64 ],
    pub black_pawn_attacks: [ u64; 64 ],
    pub knight_attacks: [ u64; 64 ],
    pub king_attacks: [ u64; 64 ],

    // Here's the magic
    pub bishop_masks: [ u64; 64 ],
    pub bishop_magics: [ u64; 64 ],
    pub bishop_attacks: Vec<Vec<u64>>,
    pub rook_masks: [ u64; 64 ],
    pub rook_magics: [ u64; 64 ],
    pub rook_attacks: Vec<Vec<u64>>,
}

impl Default for Attacks {
    fn default() -> Self {
        Attacks { white_pawn_attacks: [ 0u64; 64 ],
                  black_pawn_attacks: [ 0u64; 64 ],
                  knight_attacks: [ 0u64; 64 ],
                  king_attacks: [ 0u64; 64 ],
                  bishop_masks: [ 0u64; 64 ],
                  bishop_magics: [ 0u64; 64 ],
                  bishop_attacks: Vec::new(),
                  rook_masks: [ 0u64; 64 ],
                  rook_magics: [ 0u64; 64 ],
                  rook_attacks: Vec::new(), }
    }
}

impl Attacks {
    pub fn init( &mut self, stored: bool ) {
        self.bishop_attacks = Vec::new();
        self.rook_attacks = Vec::new();

        for pos in 0..64 {
            // Compute and store attacks for non-sliding pieces
            self.white_pawn_attacks[ pos ] = pawn_capture( pos, WHITE );
            self.black_pawn_attacks[ pos ] = pawn_capture( pos, BLACK );
            self.knight_attacks[ pos ] = knight_attack( pos );
            self.king_attacks[ pos ] = king_attack( pos );

            // Compute and store magics and attack-sets for bishops and rooks
            self.bishop_masks[ pos ] = bishop_mask( pos );
            let ( b_magic_pos, b_attacks_pos ) = magic( pos, BISHOP, stored, false );
            self.bishop_magics[ pos ] = b_magic_pos;
            self.bishop_attacks.push( b_attacks_pos );

            self.rook_masks[ pos ] = rook_mask( pos );
            let ( r_magic_pos, r_attacks_pos ) = magic( pos, ROOK, stored, false );
            self.rook_magics[ pos ] = r_magic_pos;
            self.rook_attacks.push( r_attacks_pos );
        }
    }

    pub fn r_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.rook_attacks[ pos ][ magic_hash( self.rook_magics[ pos ], self.rook_masks[ pos ] & occupancy, ROOK_SHIFTS[ pos ] ) ]
    }

    pub fn b_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.bishop_attacks[ pos ][ magic_hash( self.bishop_magics[ pos ], self.bishop_masks[ pos ] & occupancy, BISHOP_SHIFTS[ pos ] ) ]
    }
}
