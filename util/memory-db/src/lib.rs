// Copyright 2017, 2018 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Reference-counted memory-based `HashDB` implementation.

extern crate hash_db;
extern crate heapsize;
#[cfg(test)]
extern crate keccak_hasher;

use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher as KeyHasher, PlainDB, PlainDBRef};
use heapsize::HeapSizeOf;
use std::{
    collections::{hash_map::Entry, HashMap},
    hash, mem,
};

// Backing `HashMap` parametrized with a `Hasher` for the keys `Hasher::Out` and the `Hasher::StdHasher`
// as hash map builder.
type FastMap<H, T> =
    HashMap<<H as KeyHasher>::Out, T, hash::BuildHasherDefault<<H as KeyHasher>::StdHasher>>;

/// Reference-counted memory-based `HashDB` implementation.
///
/// Use `new()` to create a new database. Insert items with `insert()`, remove items
/// with `remove()`, check for existence with `contains()` and lookup a hash to derive
/// the data with `get()`. Clear with `clear()` and purge the portions of the data
/// that have no references with `purge()`.
///
/// # Example
/// ```rust
/// extern crate hash_db;
/// extern crate keccak_hasher;
/// extern crate memory_db;
///
/// use hash_db::{Hasher, HashDB};
/// use keccak_hasher::KeccakHasher;
/// use memory_db::MemoryDB;
/// fn main() {
///   let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::default();
///   let d = "Hello world!".as_bytes();
///
///   let k = m.insert(d);
///   assert!(m.contains(&k));
///   assert_eq!(m.get(&k).unwrap(), d);
///
///   m.insert(d);
///   assert!(m.contains(&k));
///
///   m.remove(&k);
///   assert!(m.contains(&k));
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
///
///   m.insert(d);
///   assert!(!m.contains(&k));

///   m.insert(d);
///   assert!(m.contains(&k));
///   assert_eq!(m.get(&k).unwrap(), d);
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
/// }
/// ```
#[derive(Clone, PartialEq)]
pub struct MemoryDB<H: KeyHasher, T> {
    data: FastMap<H, (T, i32)>,
    hashed_null_node: H::Out,
    null_node_data: T,
}

impl<'a, H, T> Default for MemoryDB<H, T>
where
    H: KeyHasher,
    T: From<&'a [u8]>,
{
    fn default() -> Self {
        Self::from_null_node(&[0u8][..], [0u8][..].into())
    }
}

impl<H, T> MemoryDB<H, T>
where
    H: KeyHasher,
    T: Default,
{
    /// Remove an element and delete it from storage if reference count reaches zero.
    /// If the value was purged, return the old value.
    pub fn remove_and_purge(&mut self, key: &<H as KeyHasher>::Out) -> Option<T> {
        if key == &self.hashed_null_node {
            return None;
        }
        match self.data.entry(key.clone()) {
            Entry::Occupied(mut entry) => {
                if entry.get().1 == 1 {
                    Some(entry.remove().0)
                } else {
                    entry.get_mut().1 -= 1;
                    None
                }
            }
            Entry::Vacant(entry) => {
                entry.insert((T::default(), -1)); // FIXME: shouldn't it be purged?
                None
            }
        }
    }
}

impl<'a, H: KeyHasher, T> MemoryDB<H, T>
where
    T: From<&'a [u8]>,
{
    /// Create a new `MemoryDB` from a given null key/data
    pub fn from_null_node(null_key: &'a [u8], null_node_data: T) -> Self {
        MemoryDB {
            data: FastMap::<H, _>::default(),
            hashed_null_node: H::hash(null_key),
            null_node_data,
        }
    }

    /// Create a new `MemoryDB` from a given null key/data
    pub fn new(data: &'a [u8]) -> Self {
        MemoryDB {
            data: FastMap::<H, _>::default(),
            hashed_null_node: H::hash(data),
            null_node_data: data.into(),
        }
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Shrinks the capacity of the map as much as possible.
    /// It will drop down as much as possible while maintaining the internal rules and possibly leaving some space in accordance with the resize policy.
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Clear all data from the database.
    ///
    /// # Examples
    /// ```rust
    /// extern crate hash_db;
    /// extern crate keccak_hasher;
    /// extern crate memory_db;
    ///
    /// use hash_db::{Hasher, HashDB};
    /// use keccak_hasher::KeccakHasher;
    /// use memory_db::MemoryDB;
    ///
    /// fn main() {
    ///   let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::default();
    ///   let hello_bytes = "Hello world!".as_bytes();
    ///   let hash = m.insert(hello_bytes);
    ///   assert!(m.contains(&hash));
    ///   m.clear();
    ///   assert!(!m.contains(&hash));
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Purge all zero-referenced data from the database.
    pub fn purge(&mut self) {
        self.data.retain(|_, &mut (_, rc)| rc != 0);
    }

    /// Return the internal map of hashes to data, clearing the current state.
    pub fn drain(&mut self) -> FastMap<H, (T, i32)> {
        mem::replace(&mut self.data, FastMap::<H, _>::default())
    }

    /// Grab the raw information associated with a key. Returns None if the key
    /// doesn't exist.
    ///
    /// Even when Some is returned, the data is only guaranteed to be useful
    /// when the refs > 0.
    pub fn raw(&self, key: &<H as KeyHasher>::Out) -> Option<(&T, i32)> {
        if key == &self.hashed_null_node {
            return Some((&self.null_node_data, 1));
        }
        self.data.get(key).map(|(value, count)| (value, *count))
    }

    /// Consolidate all the entries of `other` into `self`.
    pub fn consolidate(&mut self, mut other: Self) {
        for (key, (value, rc)) in other.drain() {
            match self.data.entry(key) {
                Entry::Occupied(mut entry) => {
                    if entry.get().1 < 0 {
                        entry.get_mut().0 = value;
                    }

                    entry.get_mut().1 += rc;
                }
                Entry::Vacant(entry) => {
                    entry.insert((value, rc));
                }
            }
        }
    }

    /// Get the keys in the database together with number of underlying references.
    pub fn keys(&self) -> HashMap<H::Out, i32> {
        self.data
            .iter()
            .filter_map(|(k, v)| if v.1 != 0 { Some((*k, v.1)) } else { None })
            .collect()
    }
}

impl<H, T> MemoryDB<H, T>
where
    H: KeyHasher,
    T: HeapSizeOf,
{
    /// Returns the size of allocated heap memory
    pub fn mem_used(&self) -> usize {
        0 //self.data.heap_size_of_children()
          // TODO Reenable above when HeapSizeOf supports arrays.
    }
}

impl<H, T> PlainDB<H::Out, T> for MemoryDB<H, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        match self.data.get(key) {
            Some(&(ref d, rc)) if rc > 0 => Some(d.clone()),
            _ => None,
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        match self.data.get(key) {
            Some(&(_, x)) if x > 0 => true,
            _ => false,
        }
    }

    fn emplace(&mut self, key: H::Out, value: T) {
        match self.data.entry(key) {
            Entry::Occupied(mut entry) => {
                let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
                if *rc <= 0 {
                    *old_value = value;
                }
                *rc += 1;
            }
            Entry::Vacant(entry) => {
                entry.insert((value, 1));
            }
        }
    }

    fn remove(&mut self, key: &H::Out) {
        match self.data.entry(*key) {
            Entry::Occupied(mut entry) => {
                let &mut (_, ref mut rc) = entry.get_mut();
                *rc -= 1;
            }
            Entry::Vacant(entry) => {
                entry.insert((T::default(), -1));
            }
        }
    }
}

impl<H, T> PlainDBRef<H::Out, T> for MemoryDB<H, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        PlainDB::get(self, key)
    }
    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self, key)
    }
}

impl<H, T> HashDB<H, T> for MemoryDB<H, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        if key == &self.hashed_null_node {
            return Some(self.null_node_data.clone());
        }

        PlainDB::get(self, key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        if key == &self.hashed_null_node {
            return true;
        }

        PlainDB::contains(self, key)
    }

    fn emplace(&mut self, key: H::Out, value: T) {
        if value == self.null_node_data {
            return;
        }

        PlainDB::emplace(self, key, value)
    }

    fn insert(&mut self, value: &[u8]) -> H::Out {
        if T::from(value) == self.null_node_data {
            return self.hashed_null_node.clone();
        }

        let key = H::hash(value);
        PlainDB::emplace(self, key.clone(), value.into());

        key
    }

    fn remove(&mut self, key: &H::Out) {
        if key == &self.hashed_null_node {
            return;
        }

        PlainDB::remove(self, key)
    }
}

impl<H, T> HashDBRef<H, T> for MemoryDB<H, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        HashDB::get(self, key)
    }
    fn contains(&self, key: &H::Out) -> bool {
        HashDB::contains(self, key)
    }
}

impl<H, T> AsPlainDB<H::Out, T> for MemoryDB<H, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, T> {
        self
    }
    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, T> {
        self
    }
}

impl<H, T> AsHashDB<H, T> for MemoryDB<H, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn as_hash_db(&self) -> &dyn HashDB<H, T> {
        self
    }
    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, T> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{HashDB, KeyHasher, MemoryDB};
    use keccak_hasher::KeccakHasher;

    #[test]
    fn memorydb_remove_and_purge() {
        let hello_bytes = b"Hello world!";
        let hello_key = KeccakHasher::hash(hello_bytes);

        let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::default();
        m.remove(&hello_key);
        assert_eq!(m.raw(&hello_key).unwrap().1, -1);
        m.purge();
        assert_eq!(m.raw(&hello_key).unwrap().1, -1);
        m.insert(hello_bytes);
        assert_eq!(m.raw(&hello_key).unwrap().1, 0);
        m.purge();
        assert_eq!(m.raw(&hello_key), None);

        let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::default();
        assert!(m.remove_and_purge(&hello_key).is_none());
        assert_eq!(m.raw(&hello_key).unwrap().1, -1);
        m.insert(hello_bytes);
        m.insert(hello_bytes);
        assert_eq!(m.raw(&hello_key).unwrap().1, 1);
        assert_eq!(&*m.remove_and_purge(&hello_key).unwrap(), hello_bytes);
        assert_eq!(m.raw(&hello_key), None);
        assert!(m.remove_and_purge(&hello_key).is_none());
    }

    #[test]
    fn consolidate() {
        let mut main = MemoryDB::<KeccakHasher, Vec<u8>>::default();
        let mut other = MemoryDB::<KeccakHasher, Vec<u8>>::default();
        let remove_key = other.insert(b"doggo");
        main.remove(&remove_key);

        let insert_key = other.insert(b"arf");
        main.emplace(insert_key, "arf".as_bytes().to_vec());

        let negative_remove_key = other.insert(b"negative");
        other.remove(&negative_remove_key); // ref cnt: 0
        other.remove(&negative_remove_key); // ref cnt: -1
        main.remove(&negative_remove_key); // ref cnt: -1

        main.consolidate(other);

        let overlay = main.drain();

        assert_eq!(
            overlay.get(&remove_key).unwrap(),
            &("doggo".as_bytes().to_vec(), 0)
        );
        assert_eq!(
            overlay.get(&insert_key).unwrap(),
            &("arf".as_bytes().to_vec(), 2)
        );
        assert_eq!(
            overlay.get(&negative_remove_key).unwrap(),
            &("negative".as_bytes().to_vec(), -2)
        );
    }

    #[test]
    fn default_works() {
        let mut db = MemoryDB::<KeccakHasher, Vec<u8>>::default();
        let hashed_null_node = KeccakHasher::hash(&[0u8][..]);
        assert_eq!(db.insert(&[0u8][..]), hashed_null_node);
    }
}
