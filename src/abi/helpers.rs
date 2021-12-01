use core::hash::Hash;

use crate::abi::AbiStore;

use dusk_hamt::Hamt;
use rkyv::{Archive, Deserialize};

struct Map<K, V> {
    wrapping: Hamt<K, V, (), AbiStore>,
}

impl<K, V> Map<K, V>
where
    K: Archive<Archived = K> + Clone + Hash + Eq,
    K: Deserialize<K, AbiStore>,
    V: Archive + Clone,
    V::Archived: Deserialize<V, AbiStore>,
{
    fn new() -> Self {
        Map {
            wrapping: Hamt::new(),
        }
    }

    fn insert(&self, key: K, val: V) {
        todo!()
    }

    // fn get(&self) -> Option<impl BranchRef<V>> {
    //     todo!()
    // }
}
