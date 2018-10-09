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

//! Generates Keccak-flavoured trie roots.

extern crate ethereum_types;
extern crate keccak_hasher;
extern crate triehash;

use ethereum_types::H256;
use keccak_hasher::KeccakHasher;

/// Generates a trie root hash for a vector of key-value tuples
pub fn trie_root<I, K, V>(input: I) -> H256
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<[u8]> + Ord,
    V: AsRef<[u8]>,
{
    triehash::trie_root::<KeccakHasher, _, _, _>(input)
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-value tuples.
pub fn sec_trie_root<I, K, V>(input: I) -> H256
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    triehash::sec_trie_root::<KeccakHasher, _, _, _>(input)
}

/// Generates a trie root hash for a vector of values
pub fn ordered_trie_root<I, V>(input: I) -> H256
where
    I: IntoIterator<Item = V>,
    V: AsRef<[u8]>,
{
    triehash::ordered_trie_root::<KeccakHasher, I>(input)
}

#[cfg(test)]
mod tests {
	use super::{trie_root, sec_trie_root, ordered_trie_root};
    use triehash;
	use keccak_hasher::KeccakHasher;

	#[test]
	fn simple_test() {
		assert_eq!(trie_root(vec![
			(b"A", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as &[u8])
		]), "d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab".into());
	}

	#[test]
	fn proxy_works() {
        let input = vec![(b"A", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as &[u8])];
		assert_eq!(
            trie_root(input.clone()),
            triehash::trie_root::<KeccakHasher, _, _, _>(input.clone())
        );

		assert_eq!(
            sec_trie_root(input.clone()),
            triehash::sec_trie_root::<KeccakHasher, _, _, _>(input.clone())
        );

        let data = &["cake", "pie", "candy"];
		assert_eq!(
            ordered_trie_root(data),
            triehash::ordered_trie_root::<KeccakHasher, _>(data)
        );
	}
}
