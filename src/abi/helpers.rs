use core::hash::Hash;

use crate::abi::AbiStore;

use dusk_hamt::{Hamt, Lookup};
use microkelvin::{BranchRef, BranchRefMut, MaybeArchived};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct Map<K, V> {
    wrapping: Hamt<K, V, (), AbiStore>,
}

impl<K, V> Map<K, V>
where
    K: Archive<Archived = K> + Clone + Hash + Eq,
    K: Deserialize<K, AbiStore>,
    V: Archive + Clone,
    V::Archived: Deserialize<V, AbiStore>,
{
    pub fn new() -> Self {
        Map {
            wrapping: Hamt::new(),
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.wrapping.insert(key, val)
    }

    pub fn get(&self, key: &K) -> Option<impl BranchRef<V>> {
        let _a = self.wrapping.get(key);
        todo!()
    }

    pub fn get_mut(&mut self, key: &K) -> Option<impl BranchRefMut<V>> {
        self.wrapping.get_mut(key);
    }
}
