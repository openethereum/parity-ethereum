// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

use std::path::Path;

use ethjson;
use trie::{TrieFactory, TrieSpec};
use ethereum_types::H256;

use super::HookType;

#[allow(dead_code)]
fn test_trie<H: FnMut(&str, HookType)>(path: &Path, json: &[u8], trie: TrieSpec, start_stop_hook: &mut H) -> Vec<String> {
	let tests = ethjson::test_helpers::trie::Test::load(json)
		.expect(&format!("Could not parse JSON trie test data from {}", path.display()));
	let factory = TrieFactory::new(trie, ethtrie::Layout);
	let mut result = vec![];

	for (name, test) in tests.into_iter() {
		start_stop_hook(&name, HookType::OnStart);

		let mut memdb = journaldb::new_memory_db();
		let mut root = H256::zero();
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

		start_stop_hook(&name, HookType::OnStop);
	}

	for i in &result {
		println!("FAILED: {}", i);
	}

	result
}

mod generic {
	use std::path::Path;
	use trie::TrieSpec;

	use super::HookType;

	#[allow(dead_code)]
	fn do_json_test<H: FnMut(&str, HookType)>(path: &Path, json: &[u8], h: &mut H) -> Vec<String> {
		super::test_trie(path, json, TrieSpec::Generic, h)
	}

	declare_test!{TrieTests_trietest, "TrieTests/trietest"}
	declare_test!{TrieTests_trieanyorder, "TrieTests/trieanyorder"}
}

mod secure {
	use std::path::Path;
	use trie::TrieSpec;

	use super::HookType;

	#[allow(dead_code)]
	fn do_json_test<H: FnMut(&str, HookType)>(path: &Path, json: &[u8], h: &mut H) -> Vec<String> {
		super::test_trie(path, json, TrieSpec::Secure, h)
	}

	declare_test!{TrieTests_hex_encoded_secure, "TrieTests/hex_encoded_securetrie_test"}
	declare_test!{TrieTests_trietest_secure, "TrieTests/trietest_secureTrie"}
	declare_test!{TrieTests_trieanyorder_secure, "TrieTests/trieanyorder_secureTrie"}
}
