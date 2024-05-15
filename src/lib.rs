use std::{cmp, collections::HashMap, hash};

use slab::Slab;

#[derive(Debug)]
pub struct AtomizedHashMap<K, T, S = hash::RandomState> {
    values: Slab<T>,
    atomizer: HashMap<K, usize, S>,
}

impl<K, T> AtomizedHashMap<K, T, hash::RandomState> {
    pub fn new() -> Self {
        Self::default()
    }
}

pub enum InsertReplaceResult {
    Inserted(usize),
    Replaced(usize),
}

impl InsertReplaceResult {
    pub const fn atom(self) -> usize {
        match self {
            InsertReplaceResult::Inserted(atom) | InsertReplaceResult::Replaced(atom) => atom,
        }
    }
}

impl<K: cmp::Eq + hash::Hash, T, S: hash::BuildHasher> AtomizedHashMap<K, T, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            values: Slab::new(),
            atomizer: HashMap::with_hasher(hash_builder),
        }
    }
    pub fn insert(&mut self, non_atomized_key: K, value: T) -> Result<usize, usize> {
        match self.atomize_key(&non_atomized_key) {
            Some(previously_inserted) => Err(previously_inserted),
            None => {
                let entry = self.values.vacant_entry();
                let atom = entry.key();
                self.atomizer.insert(non_atomized_key, atom);
                entry.insert(value);
                Ok(atom)
            }
        }
    }
    pub fn insert_replace(&mut self, non_atomized_key: K, value: T) -> InsertReplaceResult {
        match self.atomize_key(&non_atomized_key) {
            Some(atom) => {
                self.values[atom] = value;
                InsertReplaceResult::Replaced(atom)
            }
            None => {
                let entry = self.values.vacant_entry();
                let atom = entry.key();
                self.atomizer.insert(non_atomized_key, atom);
                entry.insert(value);
                InsertReplaceResult::Inserted(atom)
            }
        }
    }
    pub fn remove(&mut self, non_atomized_key: &K) -> Option<T> {
        self.atomizer
            .remove(non_atomized_key)
            .map(|atom| self.values.remove(atom))
    }
    pub fn atomize_key(&self, non_atomized_key: &K) -> Option<usize> {
        self.atomizer.get(non_atomized_key).copied()
    }
    pub fn get(&self, atom: usize) -> Option<&T> {
        self.values.get(atom)
    }
    pub fn get_mut(&mut self, atom: usize) -> Option<&mut T> {
        self.values.get_mut(atom)
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<K, T, S: Default> Default for AtomizedHashMap<K, T, S> {
    fn default() -> Self {
        Self {
            values: Slab::new(),
            atomizer: HashMap::default(),
        }
    }
}
