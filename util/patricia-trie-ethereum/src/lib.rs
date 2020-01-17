// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Façade crate for `patricia_trie` for Ethereum specific impls

pub extern crate trie_db as trie; // `pub` because we need to import this crate for the tests in `patricia_trie` and there were issues: https://gist.github.com/dvdplm/869251ee557a1b4bd53adc7c971979aa
extern crate elastic_array;
extern crate parity_bytes;
extern crate ethereum_types;
extern crate hash_db;
extern crate keccak_hasher;
extern crate rlp;

mod rlp_node_codec;

pub use rlp_node_codec::RlpNodeCodec;

use ethereum_types::H256;
use keccak_hasher::KeccakHasher;
use rlp::DecoderError;

/// Convenience type alias to instantiate a Keccak-flavoured `RlpNodeCodec`
pub type RlpCodec = RlpNodeCodec<KeccakHasher>;

#[derive(Clone, Default)]
/// Defines the working of a particular flavour of trie:
/// how keys are hashed, how values are encoded, does it use extension nodes or not.
pub struct Layout;

impl trie_db::TrieLayout for Layout {
	const USE_EXTENSION: bool = true;
	type Hash = keccak_hasher::KeccakHasher;
	type Codec = RlpNodeCodec<KeccakHasher>;
}

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDB`
///
/// Use it as a `Trie` trait object. You can use `db()` to get the backing database object.
/// Use `get` and `contains` to query values associated with keys in the trie.
///
/// # Example
/// ```
/// extern crate trie_db as trie;
/// extern crate patricia_trie_ethereum as ethtrie;
/// extern crate hash_db;
/// extern crate keccak_hasher;
/// extern crate memory_db;
/// extern crate ethereum_types;
/// extern crate elastic_array;
/// extern crate journaldb;
///
/// use trie::*;
/// use hash_db::*;
/// use keccak_hasher::KeccakHasher;
/// use memory_db::*;
/// use ethereum_types::H256;
/// use ethtrie::{TrieDB, TrieDBMut};
/// use elastic_array::ElasticArray128;
///
/// type DBValue = ElasticArray128<u8>;
///
/// fn main() {
///   let mut memdb = journaldb::new_memory_db();
///   let mut root = H256::zero();
///   TrieDBMut::new(&mut memdb, &mut root).insert(b"foo", b"bar").unwrap();
///   let t = TrieDB::new(&memdb, &root).unwrap();
///   assert!(t.contains(b"foo").unwrap());
///   assert_eq!(t.get(b"foo").unwrap().unwrap(), b"bar".to_vec());
/// }
/// ```
pub type TrieDB<'db> = trie::TrieDB<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDB`
pub type SecTrieDB<'db> = trie::SecTrieDB<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDB`
pub type FatDB<'db> = trie::FatDB<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBMut`
///
/// Use it as a `TrieMut` trait object. You can use `db()` to get the backing database object.
/// Note that changes are not committed to the database until `commit` is called.
/// Querying the root or dropping the trie will commit automatically.

/// # Example
/// ```
/// extern crate trie_db as trie;
/// extern crate patricia_trie_ethereum as ethtrie;
/// extern crate hash_db;
/// extern crate keccak_hash;
/// extern crate keccak_hasher;
/// extern crate memory_db;
/// extern crate ethereum_types;
/// extern crate elastic_array;
/// extern crate journaldb;
///
/// use keccak_hash::KECCAK_NULL_RLP;
/// use ethtrie::{TrieDBMut, trie::TrieMut};
/// use keccak_hasher::KeccakHasher;
/// use memory_db::*;
/// use ethereum_types::H256;
/// use elastic_array::ElasticArray128;
/// use trie::Trie;
///
/// type DBValue = ElasticArray128<u8>;
///
/// fn main() {
///   let mut memdb = journaldb::new_memory_db();
///   let mut root = H256::zero();
///   let mut t = TrieDBMut::new(&mut memdb, &mut root);
///   assert!(t.is_empty());
///   assert_eq!(*t.root(), KECCAK_NULL_RLP);
///   t.insert(b"foo", b"bar").unwrap();
///   assert!(t.contains(b"foo").unwrap());
///   assert_eq!(t.get(b"foo").unwrap().unwrap(), b"bar".to_vec());
///   t.remove(b"foo").unwrap();
///   assert!(!t.contains(b"foo").unwrap());
/// }
/// ```
pub type TrieDBMut<'db> = trie::TrieDBMut<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDBMut`
pub type SecTrieDBMut<'db> = trie::SecTrieDBMut<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDBMut`
pub type FatDBMut<'db> = trie::FatDBMut<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieFactory`
pub type TrieFactory = trie::TrieFactory<Layout>;

/// Convenience type alias for Keccak/Rlp flavoured trie errors
pub type TrieError = trie::TrieError<H256, DecoderError>;
/// Convenience type alias for Keccak/Rlp flavoured trie results
pub type Result<T> = trie::Result<T, H256, DecoderError>;

#[cfg(test)]
mod tests {
	use ethereum_types::H256;
	use trie::Trie;

	use crate::{TrieDB, TrieDBMut, trie::TrieMut};

	#[test]
	fn test_inline_encoding_branch() {
		let mut memdb = journaldb::new_memory_db();
		let mut root = H256::zero();
		{
			let mut triedbmut = TrieDBMut::new(&mut memdb, &mut root);
			triedbmut.insert(b"foo", b"bar").unwrap();
			triedbmut.insert(b"fog", b"b").unwrap();
			triedbmut.insert(b"fot", &vec![0u8;33][..]).unwrap();
		}
		let t = TrieDB::new(&memdb, &root).unwrap();
		assert!(t.contains(b"foo").unwrap());
		assert!(t.contains(b"fog").unwrap());
		assert_eq!(t.get(b"foo").unwrap().unwrap(), b"bar".to_vec());
		assert_eq!(t.get(b"fog").unwrap().unwrap(), b"b".to_vec());
		assert_eq!(t.get(b"fot").unwrap().unwrap(), vec![0u8;33]);
	}

	#[test]
	fn test_inline_encoding_extension() {
		let mut memdb = journaldb::new_memory_db();
		let mut root = H256::zero();
		{
			let mut triedbmut = TrieDBMut::new(&mut memdb, &mut root);
			triedbmut.insert(b"foo", b"b").unwrap();
			triedbmut.insert(b"fog", b"a").unwrap();
		}
		let t = TrieDB::new(&memdb, &root).unwrap();
		assert!(t.contains(b"foo").unwrap());
		assert!(t.contains(b"fog").unwrap());
		assert_eq!(t.get(b"foo").unwrap().unwrap(), b"b".to_vec());
		assert_eq!(t.get(b"fog").unwrap().unwrap(), b"a".to_vec());
	}

}
