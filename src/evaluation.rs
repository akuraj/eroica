//! Evaluation

use state::*;
use consts::*;
use utils::*;
use std::cmp;

// Static Evaluation Function, score from the side-to-move's POV
pub fn static_eval( state: &State ) -> i32 {
    // FIXME: Find a better way to do this.
    let pawn_pst: &[ i32 ] = &PAWN_PST;
    let knight_pst: &[ i32 ] = &KNIGHT_PST;
    let bishop_pst: &[ i32 ] = &BISHOP_PST;
    let rook_pst: &[ i32 ] = &ROOK_PST;
    let queen_pst: &[ i32 ] = &QUEEN_PST;

    let mut npm: i32 = 0;
    let mut eval_mg: i32 = 0;
    let mut eval_eg: i32 = 0;

    let mut piece_type: u8;
    let mut color: u8;
    let mut bb: u64;
    let mut pos: usize;

    for piece in ALL_PIECE_TYPES.iter() {
        piece_type = *piece & COLOR_MASK;
        color = *piece & COLOR;

        let ( piece_val_mg, piece_val_eg, pst ) : ( i32, i32, &[ i32 ] ) = match piece_type {
            PAWN => ( PAWN_VALUE_MG, PAWN_VALUE_EG, pawn_pst ),
            KNIGHT => ( KNIGHT_VALUE_MG, KNIGHT_VALUE_EG, knight_pst ),
            BISHOP => ( BISHOP_VALUE_MG, BISHOP_VALUE_EG, bishop_pst ),
            ROOK => ( ROOK_VALUE_MG, ROOK_VALUE_EG, rook_pst ),
            QUEEN => ( QUEEN_VALUE_MG, QUEEN_VALUE_EG, queen_pst ),
            _ => panic!( "Invalid piece type: {}", piece_type ),
        };

        bb = state.bit_board[ *piece ];
        match color {
            WHITE => {
                while bb != 0 {
                    if piece_type != PAWN { npm += piece_val_mg; }
                    pos = pop_lsb_pos( &mut bb );
                    eval_mg += piece_val_mg + pst[ 63 - pos ];
                    eval_eg += piece_val_eg + pst[ 63 - pos ];
                }
            },
            BLACK => {
                while bb != 0 {
                    if piece_type != PAWN { npm += piece_val_mg; }
                    pos = pop_lsb_pos( &mut bb );
                    eval_mg -= piece_val_mg + pst[ pos ];
                    eval_eg -= piece_val_eg + pst[ pos ];
                }
            },
            _ => panic!( "Invalid color: {}", color ),
        }
    }

    // Bishop Pair Bonus
    let bishop_pair_bonus = ( has_opp_color_pair( state.bit_board[ WHITE_BISHOP ] ) - has_opp_color_pair( state.bit_board[ BLACK_BISHOP ] ) ) * BISHOP_PAIR_BONUS;
    eval_mg += bishop_pair_bonus;
    eval_eg += bishop_pair_bonus;

    // Kings
    bb = state.bit_board[ WHITE_KING ];
    pos = pop_lsb_pos( &mut bb );
    eval_mg += KING_MG_PST[ 63 - pos ];
    eval_eg += KING_EG_PST[ 63 - pos ];

    bb = state.bit_board[ BLACK_KING ];
    pos = pop_lsb_pos( &mut bb );
    eval_mg -= KING_MG_PST[ pos ];
    eval_eg -= KING_EG_PST[ pos ];

    // Tapered Eval from side-to-move's POV
    npm = cmp::max( EG_NPM_LIMIT, cmp::min( MG_NPM_LIMIT, npm ) );
    let phase = ( ( npm - EG_NPM_LIMIT ) * MG_PHASE ) / ( MG_NPM_LIMIT - EG_NPM_LIMIT );
    let eval = ( phase * eval_mg + ( MG_PHASE - phase ) * eval_eg ) / MG_PHASE;
    TEMPO_BONUS + if state.to_move == WHITE { eval } else { -eval }
}
