//! MoveGen - defines an interface that provides functions to get the possible attacks/moves of pieces
//! Stores attacks for non-sliding pieces, uses magics to compute attacks for sliding pieces

use consts::*;
use utils::*;
use magics::*;

pub struct MoveGen {
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

impl MoveGen {
    pub fn new( stored: bool ) -> Self {
        let mut mg = MoveGen { knight_attacks: [ 0; 64 ],
                               king_attacks: [ 0; 64 ],
                               bishop_masks: [ 0; 64 ],
                               bishop_shifts: [ 0; 64 ],
                               bishop_magics: [ 0; 64 ],
                               bishop_offsets: [ 0; 64 ],
                               bishop_attacks: Vec::new(),
                               rook_masks: [ 0; 64 ],
                               rook_shifts: [ 0; 64 ],
                               rook_magics: [ 0; 64 ],
                               rook_offsets: [ 0; 64 ],
                               rook_attacks: Vec::new(), };

        mg.bishop_shifts = BISHOP_SHIFTS;
        mg.rook_shifts = ROOK_SHIFTS;

        mg.bishop_attacks = Vec::new();
        mg.rook_attacks = Vec::new();
        let mut b_offset: usize = 0;
        let mut r_offset: usize = 0;

        for pos in 0..64 {
            // Compute and store attacks for non-sliding pieces
            mg.knight_attacks[ pos ] = knight_attack( pos );
            mg.king_attacks[ pos ] = king_attack( pos );

            // Compute and store magics and attack-sets for bishops and rooks
            mg.bishop_masks[ pos ] = bishop_mask( pos );
            let ( b_magic_pos, b_attacks_pos ) = magic( pos, BISHOP, stored, false );
            mg.bishop_magics[ pos ] = b_magic_pos;
            mg.bishop_offsets[ pos ] = b_offset;
            mg.bishop_attacks.extend_from_slice( &b_attacks_pos );
            b_offset += b_attacks_pos.len();

            mg.rook_masks[ pos ] = rook_mask( pos );
            let ( r_magic_pos, r_attacks_pos ) = magic( pos, ROOK, stored, false );
            mg.rook_magics[ pos ] = r_magic_pos;
            mg.rook_offsets[ pos ] = r_offset;
            mg.rook_attacks.extend_from_slice( &r_attacks_pos );
            r_offset += r_attacks_pos.len();
        }

        mg
    }

    pub fn b_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.bishop_attacks[ self.bishop_offsets[ pos ] + magic_hash( self.bishop_magics[ pos ], self.bishop_masks[ pos ] & occupancy, self.bishop_shifts[ pos ] ) ]
    }

    pub fn r_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.rook_attacks[ self.rook_offsets[ pos ] + magic_hash( self.rook_magics[ pos ], self.rook_masks[ pos ] & occupancy, self.rook_shifts[ pos ] ) ]
    }

    pub fn q_moves( &self, pos: usize, occupancy: u64 ) -> u64 {
        self.b_moves( pos, occupancy ) | self.r_moves( pos, occupancy )
    }

    pub fn n_moves( &self, pos: usize ) -> u64 {
        self.knight_attacks[ pos ]
    }

    pub fn k_captures( &self, pos: usize ) -> u64 {
        self.king_attacks[ pos ]
    }

    pub fn k_moves( &self, pos: usize, color: u8, occupancy: u64, castling: u8 ) -> u64 {
        let mut moves = self.k_captures( pos );

        match ( color, pos ) {
            ( WHITE, 4 ) => {
                if ( castling & WK_CASTLE != 0 ) && ( occupancy & WKCR_OCC == 0 ) {
                    moves |= WK_CASTLE_BB;
                }

                if ( castling & WQ_CASTLE != 0 ) && ( occupancy & WQCR_OCC == 0 ) {
                    moves |= WQ_CASTLE_BB;
                }
            },
            ( BLACK, 60 ) => {
                if ( castling & BK_CASTLE != 0 ) && ( occupancy & BKCR_OCC == 0 ) {
                    moves |= BK_CASTLE_BB;
                }

                if ( castling & BQ_CASTLE != 0 ) && ( occupancy & BQCR_OCC == 0 ) {
                    moves |= BQ_CASTLE_BB;
                }
            },
            _ => {},
        }

        moves
    }

    pub fn p_moves( &self, pos: usize, color: u8, occupancy: u64 ) -> u64 {
        pawn_attack( pos, color, occupancy )
    }

    pub fn p_captures( &self, pos: usize, color: u8 ) -> u64 {
        pawn_capture( pos, color )
    }
}
