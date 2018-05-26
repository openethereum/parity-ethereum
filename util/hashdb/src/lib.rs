// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
//extern crate ethereum_types;

use std::collections::HashMap;
use elastic_array::ElasticArray128;
//use ethereum_types::H256;

use std::{fmt::Debug, hash::Hash};

pub trait Hasher: Sync + Send {
	type Out: Debug + PartialEq + Eq + Clone + Copy + Hash + Send + Sync;
	fn hash(x: &[u8]) -> Self::Out;
}
/// `HashDB` value type.
pub type DBValue = ElasticArray128<u8>;

/// Trait modelling datastore keyed by a 32-byte Keccak hash.
pub trait HashDB: Send + Sync {
	type H: Hasher;
	/// Get the keys in the database together with number of underlying references.
//	fn keys(&self) -> HashMap<H256, i32>;
	fn keys(&self) -> HashMap<<Self::H as Hasher>::Out, i32>;

	/// Look up a given hash into the bytes that hash to it, returning None if the
	/// hash is not known.
	fn get(&self, key: &<Self::H as Hasher>::Out) -> Option<DBValue>;

	/// Check for the existance of a hash-key.
	fn contains(&self, key: &<Self::H as Hasher>::Out) -> bool;

	/// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
	/// are counted and the equivalent number of `remove()`s must be performed before the data
	/// is considered dead.
	fn insert(&mut self, value: &[u8]) -> <Self::H as Hasher>::Out;

	/// Like `insert()` , except you provide the key and the data is all moved.
	fn emplace(&mut self, key: <Self::H as Hasher>::Out, value: DBValue);

	/// Remove a datum previously inserted. Insertions can be "owed" such that the same number of `insert()`s may
	/// happen without the data being eventually being inserted into the DB. It can be "owed" more than once.
	fn remove(&mut self, key: &<Self::H as Hasher>::Out);
}

///// Upcast trait.
//pub trait AsHashDB<HF:Hasher> {
//	/// Perform upcast to HashDB for anything that derives from HashDB.
//	fn as_hashdb(&self) -> &HashDB<H=HF>;
//	/// Perform mutable upcast to HashDB for anything that derives from HashDB.
//	fn as_hashdb_mut(&mut self) -> &mut HashDB<H=HF>;
//}
//
//impl<HF: Hasher, T: HashDB<H=HF>> AsHashDB<HF> for T {
//	fn as_hashdb(&self) -> &HashDB<H=HF> {
//		self
//	}
//	fn as_hashdb_mut(&mut self) -> &mut HashDB<H=HF> {
//		self
//	}
//}
//
//impl<'a> AsHashDB for &'a mut HashDB {
//	fn as_hashdb(&self) -> &HashDB {
//		&**self
//	}
//
//	fn as_hashdb_mut(&mut self) -> &mut HashDB {
//		&mut **self
//	}
//}
