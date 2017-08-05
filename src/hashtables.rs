//! Implement required hash tables here

use consts::*;
use std::fmt::Debug;

// Evaluation Type
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum EvalType {
    Alpha,
    Exact,
}

// Evaluation Result
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct Eval {
    pub eval_type: EvalType,
    pub value: i32,
}

impl Default for Eval {
    fn default() -> Self {
        Eval { eval_type: EvalType::Alpha,
               value: -INF_VALUE, }
    }
}

pub trait Hashable: Copy + Clone + Debug + Default {
    fn update( &self, new_value: Self ) -> bool;
}

impl Hashable for u64 {
    #[inline]
    fn update( &self, new_value: u64 ) -> bool {
        *self < new_value
    }
}

impl Hashable for Eval {
    #[inline]
    fn update( &self, new_value: Eval ) -> bool {
        true // Always update
    }
}

#[derive(Copy,Clone,Debug)]
pub struct HashItem<T: Hashable> {
    pub hash: u64,
    pub depth: usize,
    pub value: T,
}

impl<T: Hashable> HashItem<T> {
    #[inline]
    pub fn get( &self, hash: u64, depth: usize ) -> Option<T> {
        if self.depth == depth && self.hash == hash {
            Some( self.value )
        } else {
            None
        }
    }

    #[inline]
    pub fn set( &mut self, hash: u64, depth: usize, new_value: T ) {
        if self.depth == ERR_POS || self.value.update( new_value ) {
            self.hash = hash;
            self.depth = depth;
            self.value = new_value;
        }
    }
}

impl<T: Hashable> Default for HashItem<T> {
    fn default() -> Self {
        HashItem { hash: 0,
                   depth: ERR_POS,
                   value: Default::default(), }
    }
}

pub struct HashTable<T: Hashable> {
    pub index_mask: usize,
    pub table: Vec<HashItem<T>>,
}

impl<T: Hashable> HashTable<T> {
    pub fn new( num_bits: usize ) -> Self {
        let size: usize = 1 << num_bits;

        HashTable { index_mask: size - 1,
                    table: vec![ Default::default(); size ], }
    }

    pub fn get( &self, hash: u64, depth: usize ) -> Option<T> {
        self.table[ hash as usize & self.index_mask ].get( hash, depth )
    }

    pub fn set( &mut self, hash: u64, depth: usize, new_value: T ) {
        self.table[ hash as usize & self.index_mask ].set( hash, depth, new_value )
    }
}
