use std::hash::Hash;
use std::{borrow::Borrow, collections::HashMap, ops::Index};

pub use yapgeir_collections_macro::PersistentSlotMapKeys;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot(pub usize);

/// A collection that allows storing a heap of same-type structures
/// with a true O(1) access time.
///
/// Much like a `SlotMap` this collection returns a `Slot` on insertion,
/// which can be used to access the stored value.
///
/// Unlike `SlotMap` this collection requires you not only to define
/// a value, but also a key, and keeps a mapping from the key to a `Slot`.
///
/// This allows loading data from file, then when required finding a slot
/// for all of the keys (for example hardcoded in the code), and then
/// using the slot to reference the data.
///
/// This collection is not generational and persistent,
/// meaning that the data inserted is only cleaned when the
/// PersistentSlotMap is dropped itself.
///
/// An insert with the existing key will overwrite the data and
/// return the existing slot.
///
/// This is useful for resources which have static names and
/// a limited size, such as animations - usually they are loaded
/// from the assets and there is a limited number of them. Reloading
/// the resources without dropping the whole collection would then
/// merge assets with the existing one. All of the existing slots
/// will still remain valid in that case.
#[derive(Debug)]
pub struct PersistentSlotMap<K, V> {
    slots: Vec<V>,
    key_to_slot: HashMap<K, Slot>,
}

pub trait PersistentSlotMapKeys<K> {
    fn new<V>(slot_map: &PersistentSlotMap<K, V>) -> Self;
}

impl<K, V> Default for PersistentSlotMap<K, V> {
    fn default() -> Self {
        Self {
            slots: Default::default(),
            key_to_slot: Default::default(),
        }
    }
}

impl<K: Eq + Hash, V> PersistentSlotMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Slot {
        match self.key_to_slot.get(&key) {
            Some(&Slot(index)) => {
                self.slots[index] = value;
                Slot(index)
            }
            None => {
                let index = self.slots.len();
                self.slots.push(value);
                self.key_to_slot.insert(key, Slot(index));
                Slot(index)
            }
        }
    }

    pub fn get(&self, slot: Slot) -> Option<&V> {
        self.slots.get(slot.0)
    }

    pub fn find_slot_by_key<Q: ?Sized>(&self, key: &Q) -> Option<Slot>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.key_to_slot.get(key).cloned()
    }
}

impl<K, V> Index<Slot> for PersistentSlotMap<K, V> {
    type Output = V;

    fn index(&self, slot: Slot) -> &Self::Output {
        &self.slots[slot.0]
    }
}

impl<K: Eq + Hash, V> Index<&K> for PersistentSlotMap<K, V> {
    type Output = V;

    fn index(&self, key: &K) -> &Self::Output {
        let index = self.key_to_slot[key];
        &self.slots[index.0]
    }
}
