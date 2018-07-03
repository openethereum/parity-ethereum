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

#![feature(test)]

extern crate test;
extern crate ethcore_bytes;
extern crate ethereum_types;
extern crate memorydb;
extern crate patricia_trie as trie;
extern crate patricia_trie_ethereum as ethtrie;
extern crate keccak_hasher;
extern crate keccak_hash;
extern crate trie_standardmap;
extern crate hashdb;

use ethcore_bytes::Bytes;
use ethereum_types::H256;
use keccak_hash::keccak;
use memorydb::MemoryDB;
use test::{Bencher, black_box};
use trie::{TrieMut, Trie};
use trie_standardmap::{Alphabet, ValueMode, StandardMap};
use keccak_hasher::KeccakHasher;
use ethtrie::{TrieDB, TrieDBMut};

fn random_word(alphabet: &[u8], min_count: usize, diff_count: usize, seed: &mut H256) -> Vec<u8> {
	assert!(min_count + diff_count <= 32);
	*seed = keccak(&seed);
	let r = min_count + (seed[31] as usize % (diff_count + 1));
	let mut ret: Vec<u8> = Vec::with_capacity(r);
	for i in 0..r {
		ret.push(alphabet[seed[i] as usize % alphabet.len()]);
	}
	ret
}

fn random_bytes(min_count: usize, diff_count: usize, seed: &mut H256) -> Vec<u8> {
	assert!(min_count + diff_count <= 32);
	*seed = keccak(&seed);
	let r = min_count + (seed[31] as usize % (diff_count + 1));
	seed[0..r].to_vec()
}

fn random_value(seed: &mut H256) -> Bytes {
	*seed = keccak(&seed);
	match seed[0] % 2 {
		1 => vec![seed[31];1],
		_ => seed.to_vec(),
	}
}

#[bench]
fn trie_insertions_32_mir_1k(b: &mut Bencher) {
	let st = StandardMap {
		alphabet: Alphabet::All,
		min_key: 32,
		journal_key: 0,
		value_mode: ValueMode::Mirror,
		count: 1000,
	};
	let d = st.make();
	b.iter(&mut ||{
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
	});
}
#[bench]
fn trie_iter(b: &mut Bencher) {
	let st = StandardMap {
		alphabet: Alphabet::All,
		min_key: 32,
		journal_key: 0,
		value_mode: ValueMode::Mirror,
		count: 1000,
	};
	let d = st.make();
	let mut memdb = MemoryDB::<KeccakHasher>::new();
	let mut root = H256::new();
	{
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
	}

	b.iter(&mut ||{
		let t = TrieDB::new(&memdb, &root).unwrap();
		for n in t.iter().unwrap() {
			black_box(n).unwrap();
		}
	});
}

#[bench]
fn trie_insertions_32_ran_1k(b: &mut Bencher) {
	let st = StandardMap {
		alphabet: Alphabet::All,
		min_key: 32,
		journal_key: 0,
		value_mode: ValueMode::Random,
		count: 1000,
	};
	let d = st.make();
	let mut r = H256::new();
	b.iter(&mut ||{
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
		r = t.root().clone();
	});
}

#[bench]
fn trie_insertions_six_high(b: &mut Bencher) {
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_bytes(6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}

	b.iter(||{
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
	})
}

#[bench]
fn trie_insertions_six_mid(b: &mut Bencher) {
	let alphabet = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_";
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_word(alphabet, 6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}
	b.iter(||{
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
	})
}

#[bench]
fn trie_insertions_random_mid(b: &mut Bencher) {
	let alphabet = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_";
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_word(alphabet, 1, 5, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}

	b.iter(||{
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
	})
}

#[bench]
fn trie_insertions_six_low(b: &mut Bencher) {
	let alphabet = b"abcdef";
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_word(alphabet, 6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}

	b.iter(||{
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
	})
}
