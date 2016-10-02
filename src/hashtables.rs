//! Implement required hash tables here

use consts::*;
use std::i32;
use std::cmp::Ordering;
use std::ops::Neg;

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct HashPerftItem {
    pub hash: u64,
    pub depth: usize,
    pub perft_val: u64,
}

// Hash Table to store perft results
#[derive(Clone,Debug,PartialEq)]
pub struct HashPerft {
    pub index_mask: usize,
    pub ht: Vec<HashPerftItem>,
}

impl HashPerft {
    pub fn new( num_bits: usize ) -> Self {
        let size: usize = 1 << num_bits;

        let mut hp = HashPerft { index_mask: 0,
                                 ht: Vec::new() };

        hp.index_mask = size - 1;
        hp.ht = vec![ HashPerftItem { hash: 0, depth: ERR_POS, perft_val: 0 }; size ];

        hp
    }

    pub fn get( &self, hash: u64, depth: usize ) -> Option<u64> {
        let item = self.ht[ hash as usize & self.index_mask ];
        if item.depth == depth && item.hash == hash {
            Some( item.perft_val )
        } else {
            None
        }
    }

    pub fn set( &mut self, hash: u64, depth: usize, perft_val: u64 ) {
        let item: &mut HashPerftItem = &mut self.ht[ hash as usize & self.index_mask ];

        if item.depth == ERR_POS || item.perft_val < perft_val {
            item.hash = hash;
            item.depth = depth;
            item.perft_val = perft_val;
        }
    }
}

// Evaluation Type
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum EvalType {
    Alpha,
    Exact,
    Beta,
}

impl Neg for EvalType {
    type Output = EvalType;

    fn neg( self ) -> EvalType {
        match self {
            EvalType::Alpha => EvalType::Beta,
            EvalType::Exact => EvalType::Exact,
            EvalType::Beta => EvalType::Alpha,
        }
    }
}

// Evaluation Result
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct Eval {
    pub eval_type: EvalType,
    pub value: i32,
}

impl PartialOrd for Eval {
    fn partial_cmp( &self, other: &Eval ) -> Option<Ordering> {
        match ( self.eval_type, other.eval_type ) {
            ( EvalType::Alpha, EvalType::Alpha ) => None,
            ( EvalType::Alpha, EvalType::Exact ) => if self.value > other.value { Some( Ordering::Greater ) } else { None },
            ( EvalType::Alpha, EvalType::Beta ) => if self.value > other.value { Some( Ordering::Greater ) } else { None },
            ( EvalType::Exact, EvalType::Alpha ) => if self.value < other.value { Some( Ordering::Less ) } else { None },
            ( EvalType::Exact, EvalType::Exact ) => Some( self.value.cmp( &other.value ) ),
            ( EvalType::Exact, EvalType::Beta ) => if self.value > other.value { Some( Ordering::Greater ) } else { None },
            ( EvalType::Beta, EvalType::Alpha ) => if self.value < other.value { Some( Ordering::Less ) } else { None },
            ( EvalType::Beta, EvalType::Exact ) => if self.value < other.value { Some( Ordering::Less ) } else { None },
            ( EvalType::Beta, EvalType::Beta ) => None,
        }
    }
}

impl Neg for Eval {
    type Output = Eval;

    fn neg( self ) -> Eval {
        Eval { eval_type: -self.eval_type,
               value: -self.value, }
    }
}

// Transposition Table entry
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct TTItem {
    pub hash: u64,
    pub depth: usize,
    pub eval: Eval,
}

// TranspositionTable
#[derive(Clone,Debug,PartialEq)]
pub struct TranspositionTable {
    pub index_mask: usize,
    pub table: Vec<TTItem>,
}

impl TranspositionTable {
    pub fn new( num_bits: usize ) -> Self {
        let size: usize = 1 << num_bits;

        let dummy_tt_item = TTItem { hash: 0,
                                     depth: ERR_POS,
                                     eval: Eval { eval_type: EvalType::Alpha,
                                                  value: i32::MIN, }, };

        TranspositionTable { index_mask: size - 1,
                             table: vec![ dummy_tt_item; size ], }
    }

    pub fn get( &self, hash: u64, depth: usize ) -> Option<Eval> {
        let item = self.table[ hash as usize & self.index_mask ];

        if item.depth == depth && item.hash == hash {
            Some( item.eval )
        } else {
            None
        }
    }

    pub fn set( &mut self, hash: u64, depth: usize, eval: Eval ) {
        let item: &mut TTItem = &mut self.table[ hash as usize & self.index_mask ];

        if item.depth == ERR_POS || item.depth < depth {
            item.hash = hash;
            item.depth = depth;
            item.eval = eval;
        }
    }
}
