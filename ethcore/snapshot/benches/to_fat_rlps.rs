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

//! Benchmark snapshot::account::to_fat_rlps() which is a hot call during snapshots.

use std::collections::HashSet;
use std::str::FromStr;

use account_db::AccountDB;
use common_types::{
	basic_account::BasicAccount,
	snapshot::Progress
};
use criterion::{Criterion, criterion_group, criterion_main, black_box};
use ethcore::test_helpers::new_temp_db;
use ethereum_types::H256;
use snapshot::test_helpers::to_fat_rlps;
use tempdir::TempDir;
use ethtrie::TrieDB;
use trie_db::Trie;

fn fat_rlps(c: &mut Criterion) {
	let tempdir = TempDir::new("").unwrap();
	let blockchain_db = new_temp_db(tempdir.path());

	let mut state_rebuilder = snapshot::StateRebuilder::new(blockchain_db.key_value().clone(), journaldb::Algorithm::OverlayRecent);

	//	Data dumped off Ropsten using a custom build on 2019-10-21:  Snapshot Worker #2 - State TRACE snapshot  BAIL with account_key_hash: 0x2032dfb6ad93f1928dac70627a8767d2232568a1a7bf1c91ea416988000f8275 first_chunk_size: 4194304 max_chunk_size: 4194304 part: Some(2) root: 0xbc6a2ecc1ece1716dc4c249af4bd5ff5437eb850b87ebebea9bcba3700439197 fat_rlp len: 5279
	let chunk = include_bytes!("./state-chunk-5279-0x2032dfb6ad93f1928dac70627a8767d2232568a1a7bf1c91ea416988000f8275.rlp").to_vec();
	let flag = std::sync::atomic::AtomicBool::new(true);

	state_rebuilder.feed(&chunk, &flag).expect("feed fail");
	let state_root = state_rebuilder.state_root();
	let journal_db = state_rebuilder.finalize(123, H256::random()).expect("finalize fail");
	let hashdb = journal_db.as_hash_db();

	let account_trie = TrieDB::new(&hashdb, &state_root).expect("triedb has our root");
	let mut account_iter = account_trie.iter().expect("there's a root in our trie");

	let (account_key, account_data) = account_iter.next().expect("there is data in the trie").expect("…for real");
	let account_hash = H256::from_slice(&account_key);
	assert_eq!(account_hash, H256::from_str("2032dfb6ad93f1928dac70627a8767d2232568a1a7bf1c91ea416988000f8275").unwrap());

	let basic_account: BasicAccount = rlp::decode(&*account_data).expect("rlp from disk is ok");
	let account_db = AccountDB::from_hash(hashdb, account_hash);

	let progress = Progress::new();
	let mut used_code = HashSet::new();

	c.bench_function("Ropsten 5kb fat_rlp", |b| {
		b.iter(|| {
			let _ = to_fat_rlps(
				black_box(&account_hash),
				black_box(&basic_account),
				black_box(&account_db),
				black_box(&mut used_code),
				black_box(4194304),
				black_box(4194304),
				&progress
			);
		})
	});
}


criterion_group!(benches, fat_rlps);
criterion_main!(benches);
