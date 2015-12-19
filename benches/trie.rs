#![feature(test)]

extern crate test;
extern crate rand;
extern crate ethcore_util;
#[macro_use]
extern crate log;

use test::Bencher;
use ethcore_util::hash::*;
use ethcore_util::bytes::*;
use ethcore_util::trie::*;
use ethcore_util::hashdb::*;
use ethcore_util::memorydb::*;
use ethcore_util::triehash::*;
use ethcore_util::sha3::*;


fn random_word(alphabet: &[u8], min_count: usize, diff_count: usize, seed: &mut H256) -> Vec<u8> {
	assert!(min_count + diff_count <= 32);
	*seed = seed.sha3();
	let r = min_count + (seed.bytes()[31] as usize % (diff_count + 1));
	let mut ret: Vec<u8> = Vec::with_capacity(r);
	for i in 0..r {
		ret.push(alphabet[seed.bytes()[i] as usize % alphabet.len()]);
	}
	ret
}

fn random_bytes(min_count: usize, diff_count: usize, seed: &mut H256) -> Vec<u8> {
	assert!(min_count + diff_count <= 32);
	*seed = seed.sha3();
	let r = min_count + (seed.bytes()[31] as usize % (diff_count + 1));
	seed.bytes()[0..r].to_vec()
}

fn random_value(seed: &mut H256) -> Bytes {
	*seed = seed.sha3();
	match seed.bytes()[0] % 2 {
		1 => vec![seed.bytes()[31];1],
		_ => seed.bytes().to_vec(),
	}
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
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1);
		}
	})
}

#[bench]
fn triehash_insertions_six_high(b: &mut Bencher) {
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_bytes(6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}

	b.iter(&||{
		trie_root(d.clone());
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
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1);
		}
		debug!("hash_count={:?}", t.hash_count);
	})
}

#[bench]
fn triehash_insertions_six_mid(b: &mut Bencher) {
	let alphabet = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_";
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_word(alphabet, 6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}
	b.iter(||{
		trie_root(d.clone());
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
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1);
		}
	})
}

#[bench]
fn triehash_insertions_random_mid(b: &mut Bencher) {
	let alphabet = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_";
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_word(alphabet, 1, 5, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}

	b.iter(||{
		trie_root(d.clone());
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
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1);
		}
	})
}

#[bench]
fn triehash_insertions_six_low(b: &mut Bencher) {
	let alphabet = b"abcdef";
	let mut d: Vec<(Bytes, Bytes)> = Vec::new();
	let mut seed = H256::new();
	for _ in 0..1000 {
		let k = random_word(alphabet, 6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}

	b.iter(||{
		trie_root(d.clone());
	})
}

#[bench]
fn sha3x1000(b: &mut Bencher) {
	b.iter(||{
		let mut seed = H256::new();
		for _ in 0..1000 {
			seed = seed.sha3()
		}
	})
}
