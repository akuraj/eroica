//! Attacks - defines an interface that provides functions to get the possible attacks/moves of pieces
//! Stores attacks for non-sliding pieces, uses magics to compute attacks for sliding pieces

use std::default::Default;
use consts::*;
use utils::*;
use magics::*;

pub struct Attacks {
    pub knight_attacks: [ u64; 64 ],
    pub king_attacks: [ u64; 64 ],

    // Here's the magic
    pub bishop_masks: [ u64; 64 ],
    pub bishop_shifts: [ u8; 64 ],
    pub bishop_magics: [ u64; 64 ],
    pub bishop_offsets: [ usize; 64 ],
    pub bishop_attacks: Vec<u64>,
    pub rook_masks: [ u64; 64 ],
    pub rook_shifts: [ u8; 64 ],
    pub rook_magics: [ u64; 64 ],
    pub rook_offsets: [ usize; 64 ],
    pub rook_attacks: Vec<u64>,
}

impl Default for Attacks {
    fn default() -> Self {
        Attacks { knight_attacks: [ 0u64; 64 ],
                  king_attacks: [ 0u64; 64 ],
                  bishop_masks: [ 0u64; 64 ],
                  bishop_shifts: [ 0u8; 64 ],
                  bishop_magics: [ 0u64; 64 ],
                  bishop_offsets: [ 0usize; 64 ],
                  bishop_attacks: Vec::new(),
                  rook_masks: [ 0u64; 64 ],
                  rook_shifts: [ 0u8; 64 ],
                  rook_magics: [ 0u64; 64 ],
                  rook_offsets: [ 0usize; 64 ],
                  rook_attacks: Vec::new(), }
    }
}

impl Attacks {
    pub fn init( &mut self, stored: bool ) {
        self.bishop_shifts = BISHOP_SHIFTS;
        self.rook_shifts = ROOK_SHIFTS;

        self.bishop_attacks = Vec::new();
        self.rook_attacks = Vec::new();
        let mut b_offset: usize = 0;
        let mut r_offset: usize = 0;

        for pos in 0..64 {
            // Compute and store attacks for non-sliding pieces
            self.knight_attacks[ pos ] = knight_attack( pos );
            self.king_attacks[ pos ] = king_attack( pos );

            // Compute and store magics and attack-sets for bishops and rooks
            self.bishop_masks[ pos ] = bishop_mask( pos );
            let ( b_magic_pos, b_attacks_pos ) = magic( pos, BISHOP, stored, false );
            self.bishop_magics[ pos ] = b_magic_pos;
            self.bishop_offsets[ pos ] = b_offset;
            self.bishop_attacks.extend_from_slice( &b_attacks_pos );
            b_offset += b_attacks_pos.len();

            self.rook_masks[ pos ] = rook_mask( pos );
            let ( r_magic_pos, r_attacks_pos ) = magic( pos, ROOK, stored, false );
            self.rook_magics[ pos ] = r_magic_pos;
            self.rook_offsets[ pos ] = r_offset;
            self.rook_attacks.extend_from_slice( &r_attacks_pos );
            r_offset += r_attacks_pos.len();
        }
    }

    pub fn b_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.bishop_attacks[ self.bishop_offsets[ pos ] + magic_hash( self.bishop_magics[ pos ], self.bishop_masks[ pos ] & occupancy, self.bishop_shifts[ pos ] ) ]
    }

    pub fn r_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.rook_attacks[ self.rook_offsets[ pos ] + magic_hash( self.rook_magics[ pos ], self.rook_masks[ pos ] & occupancy, self.rook_shifts[ pos ] ) ]
    }

    pub fn q_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.bishop_attacks[ self.bishop_offsets[ pos ] + magic_hash( self.bishop_magics[ pos ], self.bishop_masks[ pos ] & occupancy, self.bishop_shifts[ pos ] ) ] |
        self.rook_attacks[ self.rook_offsets[ pos ] + magic_hash( self.rook_magics[ pos ], self.rook_masks[ pos ] & occupancy, self.rook_shifts[ pos ] ) ]
    }

    pub fn n_moves( &self, pos: usize ) -> u64 {
        self.knight_attacks[ pos ]
    }

    pub fn k_moves( &self, pos: usize ) -> u64 {
        self.king_attacks[ pos ]
    }
}
