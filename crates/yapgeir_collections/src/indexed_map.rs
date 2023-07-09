use std::hash::Hash;
use std::{borrow::Borrow, collections::HashMap, ops::Index};

#[macro_export]
macro_rules! indexed_map_key_index {
    ($struct_name:ident<$map_key:ty>, [ $($field:ident : $key:expr),* ]) => {
        pub struct $struct_name {
            pub $( $field: usize ),*
        }

        impl $struct_name {
            pub fn new<V>(map: &IndexedMap<$map_key, V>) -> Self {
                $(
                let $field = map.get_index($key)
                    .expect(concat!(stringify!($key), " not found in IndexedMap"));
                )*
                Self {
                    $( $field ),*
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct IndexedMap<K, V> {
    index_to_value: Vec<V>,
    key_to_index: HashMap<K, usize>,
}

impl<K, V> Default for IndexedMap<K, V> {
    fn default() -> Self {
        Self {
            index_to_value: Default::default(),
            key_to_index: Default::default(),
        }
    }
}

impl<K: Eq + Hash, V> IndexedMap<K, V> {
    pub fn capacity(&self) -> usize {
        self.index_to_value.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.index_to_value.reserve(additional);
        self.key_to_index.reserve(additional);
    }

    pub fn insert(&mut self, key: K, value: V) -> usize {
        match self.key_to_index.get(&key) {
            Some(&index) => {
                self.index_to_value[index] = value;
                index
            }
            None => {
                let index = self.index_to_value.len();
                self.index_to_value.push(value);
                self.key_to_index.insert(key, index);
                index
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&V> {
        self.index_to_value.get(index)
    }

    pub fn get_index<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.key_to_index.get(key).cloned()
    }
}

impl<K, V> Index<usize> for IndexedMap<K, V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        &self.index_to_value[index]
    }
}

impl<K: Eq + Hash, V> Index<&K> for IndexedMap<K, V> {
    type Output = V;

    fn index(&self, key: &K) -> &Self::Output {
        let index = self.key_to_index[key];
        &self.index_to_value[index]
    }
}
