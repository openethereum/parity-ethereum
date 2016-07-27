// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Key-value datastore with a modified Merkle tree.
extern crate rand;

use bytes::*;
use sha3::*;
use hash::*;
use rlp::encode;

/// Alphabet to use when creating words for insertion into tries.
pub enum Alphabet {
	/// All values are allowed in each bytes of the key.
	All,
	/// Only a 6 values ('a' - 'f') are chosen to compose the key.
	Low,
	/// Quite a few values (around 32) are chosen to compose the key.
	Mid,
	/// A set of bytes given is used to compose the key.
	Custom(Bytes),
}

/// Means of determining the value.
pub enum ValueMode {
	/// Same as the key.
	Mirror,
	/// Randomly (50:50) 1 or 32 byte randomly string.
	Random,
	/// RLP-encoded index.
	Index,
}

/// Standard test map for profiling tries.
pub struct StandardMap {
	/// The alphabet to use for keys.
	pub alphabet: Alphabet,
	/// Minimum size of key.
	pub min_key: usize,
	/// Delta size of key.
	pub journal_key: usize,
	/// Mode of value generation.
	pub value_mode: ValueMode,
	/// Number of keys.
	pub count: usize,
}

impl StandardMap {
	/// Get a bunch of random bytes, at least `min_count` bytes, at most `min_count` + `journal_count` bytes.
	/// `seed` is mutated pseudoramdonly and used.
	fn random_bytes(min_count: usize, journal_count: usize, seed: &mut H256) -> Vec<u8> {
		assert!(min_count + journal_count <= 32);
		*seed = seed.sha3();
		let r = min_count + (seed[31] as usize % (journal_count + 1));
		seed[0..r].to_vec()
	}

	/// Get a random value. Equal chance of being 1 byte as of 32. `seed` is mutated pseudoramdonly and used.
	fn random_value(seed: &mut H256) -> Bytes {
		*seed = seed.sha3();
		match seed[0] % 2 {
			1 => vec![seed[31];1],
			_ => seed.to_vec(),
		}
	}

	/// Get a random word of, at least `min_count` bytes, at most `min_count` + `journal_count` bytes.
	/// Each byte is an item from `alphabet`. `seed` is mutated pseudoramdonly and used.
	fn random_word(alphabet: &[u8], min_count: usize, journal_count: usize, seed: &mut H256) -> Vec<u8> {
		assert!(min_count + journal_count <= 32);
		*seed = seed.sha3();
		let r = min_count + (seed[31] as usize % (journal_count + 1));
		let mut ret: Vec<u8> = Vec::with_capacity(r);
		for i in 0..r {
			ret.push(alphabet[seed[i] as usize % alphabet.len()]);
		}
		ret
	}

	/// Create the standard map (set of keys and values) for the object's fields.
	pub fn make(&self) -> Vec<(Bytes, Bytes)> {
		self.make_with(&mut H256::new())
	}

	/// Create the standard map (set of keys and values) for the object's fields, using the given seed.
	pub fn make_with(&self, seed: &mut H256) -> Vec<(Bytes, Bytes)> {
		let low = b"abcdef";
		let mid = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_";

		let mut d: Vec<(Bytes, Bytes)> = Vec::new();
		for index in 0..self.count {
			let k = match self.alphabet {
				Alphabet::All => Self::random_bytes(self.min_key, self.journal_key, seed),
				Alphabet::Low => Self::random_word(low, self.min_key, self.journal_key, seed),
				Alphabet::Mid => Self::random_word(mid, self.min_key, self.journal_key, seed),
				Alphabet::Custom(ref a) => Self::random_word(a, self.min_key, self.journal_key, seed),
			};
			let v = match self.value_mode {
				ValueMode::Mirror => k.clone(),
				ValueMode::Random => Self::random_value(seed),
				ValueMode::Index => encode(&index).to_vec(),
			};
			d.push((k, v))
		}
		d
	}
}
