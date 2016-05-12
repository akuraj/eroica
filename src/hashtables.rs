//! Implement required hash tables here

use consts::*;

#[derive(Copy,Clone,Debug,PartialEq)]
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
        let item = self.ht[ ( hash as usize ) & self.index_mask ];
        if item.depth == depth && item.hash == hash {
            Some( item.perft_val )
        } else {
            None
        }
    }

    pub fn set( &mut self, hash: u64, depth: usize, perft_val: u64 ) {
        let item: &mut HashPerftItem = &mut self.ht[ ( hash as usize ) & self.index_mask ];

        if item.depth == ERR_POS || item.perft_val < perft_val {
            item.hash = hash;
            item.depth = depth;
            item.perft_val = perft_val;
        }
    }
}
