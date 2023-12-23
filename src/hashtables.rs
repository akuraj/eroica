//! Implement required hash tables here

use crate::consts::*;
use std::fmt::Debug;

// Shorthand to represent the combination of a few traits.
pub trait HashReq: Copy + Clone + Debug + Default {}
impl<T: Copy + Clone + Debug + Default> HashReq for T {}

#[derive(Copy, Clone, Debug)]
pub struct HashItem<T: HashReq> {
    pub hash: u64,
    pub depth: usize,
    pub value: T,
}

// This trait provides an interface to decide when to update the HashItem.
pub trait UpdateHash<T: HashReq> {
    fn update(&self, depth: usize, value: T) -> bool;
}

impl<T: HashReq> UpdateHash<T> for HashItem<T> {
    #[inline]
    default fn update(&self, _depth: usize, _value: T) -> bool {
        true // Always update
    }
}

impl UpdateHash<u64> for HashItem<u64> {
    #[inline]
    fn update(&self, _depth: usize, value: u64) -> bool {
        self.depth == ERR_POS || self.value < value
    }
}

impl<T: HashReq> HashItem<T> {
    #[inline]
    pub fn get(&self, hash: u64, depth: usize) -> Option<T> {
        if self.depth == depth && self.hash == hash {
            Some(self.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, hash: u64, depth: usize, new_value: T) {
        if self.update(depth, new_value) {
            self.hash = hash;
            self.depth = depth;
            self.value = new_value;
        }
    }
}

impl<T: HashReq> Default for HashItem<T> {
    fn default() -> Self {
        HashItem {
            hash: 0,
            depth: ERR_POS,
            value: Default::default(),
        }
    }
}

pub struct HashTable<T: HashReq> {
    pub index_mask: usize,
    pub table: Vec<HashItem<T>>,
}

impl<T: HashReq> HashTable<T> {
    pub fn new(num_bits: usize) -> Self {
        let size: usize = 1 << num_bits;

        HashTable {
            index_mask: size - 1,
            table: vec![Default::default(); size],
        }
    }

    pub fn get(&self, hash: u64, depth: usize) -> Option<T> {
        self.table[hash as usize & self.index_mask].get(hash, depth)
    }

    pub fn set(&mut self, hash: u64, depth: usize, new_value: T) {
        self.table[hash as usize & self.index_mask].set(hash, depth, new_value)
    }
}
