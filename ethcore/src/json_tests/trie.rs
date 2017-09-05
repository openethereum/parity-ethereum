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

use ethjson;
use util::trie::{TrieFactory, TrieSpec};
use bigint::hash::H256;
use util::memorydb::MemoryDB;

fn test_trie(json: &[u8], trie: TrieSpec) -> Vec<String> {
	let tests = ethjson::trie::Test::load(json).unwrap();
	let factory = TrieFactory::new(trie);
	let mut result = vec![];

	for (name, test) in tests.into_iter() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::default();
		let mut t = factory.create(&mut memdb, &mut root);

		for (key, value) in test.input.data.into_iter() {
			let key: Vec<u8> = key.into();
			let value: Vec<u8> = value.map_or_else(Vec::new, Into::into);
			t.insert(&key, &value)
				.expect(&format!("Trie test '{:?}' failed due to internal error", name));
		}

		if *t.root() != test.root.into() {
			result.push(format!("Trie test '{:?}' failed.", name));
		}
	}

	for i in &result {
		println!("FAILED: {}", i);
	}

	result
}

mod generic {
	use util::trie::TrieSpec;

	fn do_json_test(json: &[u8]) -> Vec<String> {
		super::test_trie(json, TrieSpec::Generic)
	}

	declare_test!{TrieTests_trietest, "TrieTests/trietest"}
	declare_test!{TrieTests_trieanyorder, "TrieTests/trieanyorder"}
}

mod secure {
	use util::trie::TrieSpec;

	fn do_json_test(json: &[u8]) -> Vec<String> {
		super::test_trie(json, TrieSpec::Secure)
	}

	declare_test!{TrieTests_hex_encoded_secure, "TrieTests/hex_encoded_securetrie_test"}
	declare_test!{TrieTests_trietest_secure, "TrieTests/trietest_secureTrie"}
	declare_test!{TrieTests_trieanyorder_secure, "TrieTests/trieanyorder_secureTrie"}
}
