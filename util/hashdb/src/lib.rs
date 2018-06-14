// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Database of byte-slices keyed to their Keccak hash.
extern crate elastic_array;
extern crate ethereum_types;
extern crate heapsize;
extern crate tiny_keccak;

use elastic_array::ElasticArray128;
use ethereum_types::H256;
use heapsize::HeapSizeOf;
use std::collections::HashMap;
use std::{fmt::Debug, hash::Hash};
use tiny_keccak::Keccak;

pub trait Hasher: Sync + Send {
	type Out: AsRef<[u8]> + Debug + PartialEq + Eq + Clone + Copy + Hash + Send + Sync /* REVIEW: how do I get around this? --> */ + HeapSizeOf;
	const HASHED_NULL_RLP: Self::Out;
	fn hash(x: &[u8]) -> Self::Out;
}

// REVIEW: Where do the concrete Hasher implementations go? Own crate?
#[derive(Default, Debug, Clone, PartialEq)]
pub struct KeccakHasher;
impl Hasher for KeccakHasher {
	type Out = H256;
	const HASHED_NULL_RLP: H256 = H256( [0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21] );

	fn hash(x: &[u8]) -> Self::Out {
		let mut out = [0;32];
		Keccak::keccak256(x, &mut out);
		out.into()
	}
}
/// Convenience type for crates that need a `HashDB` with Keccak hashes
pub type KeccakHashDB = HashDB<KeccakHasher>;

/// `HashDB` value type.
pub type DBValue = ElasticArray128<u8>;

/// Trait modelling datastore keyed by a 32-byte Keccak hash.
pub trait HashDB<H: Hasher>: Send + Sync + AsHashDB<H> {
	/// Get the keys in the database together with number of underlying references.
	fn keys(&self) -> HashMap<H::Out, i32>;

	/// Look up a given hash into the bytes that hash to it, returning None if the
	/// hash is not known.
	fn get(&self, key: &H::Out) -> Option<DBValue>;

	/// Check for the existance of a hash-key.
	fn contains(&self, key: &H::Out) -> bool;

	/// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
	/// are counted and the equivalent number of `remove()`s must be performed before the data
	/// is considered dead.
	fn insert(&mut self, value: &[u8]) -> H::Out;

	/// Like `insert()`, except you provide the key and the data is all moved.
	fn emplace(&mut self, key: H::Out, value: DBValue);

	/// Remove a datum previously inserted. Insertions can be "owed" such that the same number of `insert()`s may
	/// happen without the data being eventually being inserted into the DB. It can be "owed" more than once.
	fn remove(&mut self, key: &H::Out);
}

/// Upcast trait.
pub trait AsHashDB<H: Hasher> {
	/// Perform upcast to HashDB for anything that derives from HashDB.
	fn as_hashdb(&self) -> &HashDB<H>;
	/// Perform mutable upcast to HashDB for anything that derives from HashDB.
	fn as_hashdb_mut(&mut self) -> &mut HashDB<H>;
}

// TODO: This conflicts with an impl `for T`, see https://stackoverflow.com/questions/48432842/implementing-a-trait-for-reference-and-non-reference-types-causes-conflicting-im
impl<'a, H: Hasher> AsHashDB<H> for &'a mut HashDB<H> {
	fn as_hashdb(&self) -> &HashDB<H> { &**self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<H> { &mut **self }
}

