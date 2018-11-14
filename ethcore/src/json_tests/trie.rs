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

use ethjson;
use trie::{TrieFactory, TrieSpec};
use ethtrie::RlpCodec;
use ethereum_types::H256;
use memorydb::MemoryDB;
use keccak_hasher::KeccakHasher;
use kvdb::DBValue;

use super::HookType;

pub use self::generic::run_test_path as run_generic_test_path;
pub use self::generic::run_test_file as run_generic_test_file;
pub use self::secure::run_test_path as run_secure_test_path;
pub use self::secure::run_test_file as run_secure_test_file;

fn test_trie<H: FnMut(&str, HookType)>(json: &[u8], trie: TrieSpec, start_stop_hook: &mut H) -> Vec<String> {
	let tests = ethjson::trie::Test::load(json).unwrap();
	let factory = TrieFactory::<_, RlpCodec>::new(trie);
	let mut result = vec![];

	for (name, test) in tests.into_iter() {
		start_stop_hook(&name, HookType::OnStart);

		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
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

	/// Run generic trie jsontests on a given folder.
	pub fn run_test_path<H: FnMut(&str, HookType)>(p: &Path, skip: &[&'static str], h: &mut H) {
		::json_tests::test_common::run_test_path(p, skip, do_json_test, h)
	}

	/// Run generic trie jsontests on a given file.
	pub fn run_test_file<H: FnMut(&str, HookType)>(p: &Path, h: &mut H) {
		::json_tests::test_common::run_test_file(p, do_json_test, h)
	}

	fn do_json_test<H: FnMut(&str, HookType)>(json: &[u8], h: &mut H) -> Vec<String> {
		super::test_trie(json, TrieSpec::Generic, h)
	}

	declare_test!{TrieTests_trietest, "TrieTests/trietest"}
	declare_test!{TrieTests_trieanyorder, "TrieTests/trieanyorder"}
}

mod secure {
	use std::path::Path;
	use trie::TrieSpec;

	use super::HookType;

	/// Run secure trie jsontests on a given folder.
	pub fn run_test_path<H: FnMut(&str, HookType)>(p: &Path, skip: &[&'static str], h: &mut H) {
		::json_tests::test_common::run_test_path(p, skip, do_json_test, h)
	}

	/// Run secure trie jsontests on a given file.
	pub fn run_test_file<H: FnMut(&str, HookType)>(p: &Path, h: &mut H) {
		::json_tests::test_common::run_test_file(p, do_json_test, h)
	}

	fn do_json_test<H: FnMut(&str, HookType)>(json: &[u8], h: &mut H) -> Vec<String> {
		super::test_trie(json, TrieSpec::Secure, h)
	}

	declare_test!{TrieTests_hex_encoded_secure, "TrieTests/hex_encoded_securetrie_test"}
	declare_test!{TrieTests_trietest_secure, "TrieTests/trietest_secureTrie"}
	declare_test!{TrieTests_trieanyorder_secure, "TrieTests/trieanyorder_secureTrie"}
}
