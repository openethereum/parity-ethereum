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

//! Benchmark snapshot::account::to_fat_rlps() which is a hot call during snapshots.

use std::collections::HashSet;

use account_db::AccountDB;
use common_types::{
	basic_account::BasicAccount,
	snapshot::Progress
};
use criterion::{Criterion, criterion_group, criterion_main, black_box};
use ethcore::test_helpers::new_temp_db;
use ethereum_types::H256;
use parking_lot::RwLock;
use snapshot::test_helpers::to_fat_rlps;
use tempdir::TempDir;
use ethtrie::TrieDB;
use trie_db::Trie;

fn fat_rlps(c: &mut Criterion) {
	let tempdir = TempDir::new("").unwrap();
	let blockchain_db = new_temp_db(tempdir.path());

	let mut state_rebuilder = snapshot::StateRebuilder::new(blockchain_db.key_value().clone(), journaldb::Algorithm::OverlayRecent);

	// Chunk data collected from mainnet/ropsten around blocks 8.7M/6.8M (end of Oct '19). The data
	// sizes represent roughly the 99-percentile of account sizes. It takes some effort to find
	// accounts of representative size that are self-contained (i.e. do not have code in other
	// chunks).
	let chunks = vec![
		// Ropsten
		include_bytes!("./state-chunk-5279-0x2032dfb6ad93f1928dac70627a8767d2232568a1a7bf1c91ea416988000f8275.rlp").to_vec(),
		// Ropsten
		include_bytes!("./state-chunk-5905-0x104ff12a3fda9e0cb1aeef41fe7092982134eb116292c0eec725c32a815ef0ea.rlp").to_vec(),
		// Ropsten
		include_bytes!("./state-chunk-6341-0x3042ea62f982fd0cea02847ff0fd103a0beef3bb19389f5e77113c3ea355f803.rlp").to_vec(),
		// Ropsten
		include_bytes!("./state-chunk-6720-0x2075481dccdc2c4419112bfea2d09219a7223614656722a1a05a930baf2b0dd7.rlp").to_vec(),
		// Mainnet
		include_bytes!("./state-chunk-6933-0x104102770901b53230e78cfc8f6edce282eb21bfa00aa1c3543c79cb3402cf2d.rlp").to_vec(),
	];

	let flag = std::sync::atomic::AtomicBool::new(true);
	for chunk in &chunks {
		state_rebuilder.feed(&chunk, &flag).expect("feed fail");
	}
	let state_root = state_rebuilder.state_root();
	let journal_db = state_rebuilder.finalize(123, H256::random()).expect("finalize fail");
	let hashdb = journal_db.as_hash_db();
	let account_trie = TrieDB::new(&hashdb, &state_root).expect("triedb has our root");
	let account_iter = account_trie.iter().expect("there's a root in our trie");

	for (idx, item) in account_iter.enumerate() {
		let (account_key, account_data) = item.expect("data is the db is ok");
		let account_hash = H256::from_slice(&account_key);
		let basic_account: BasicAccount = rlp::decode(&*account_data).expect("rlp from disk is ok");
		let account_db = AccountDB::from_hash(hashdb, account_hash);
		let progress = RwLock::new(Progress::new());
		let mut used_code = HashSet::new();

		let bench_name = format!("to_fat_rlps, {} bytes, ({})", chunks[idx].len(), account_hash);
		c.bench_function(&bench_name, |b| {
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
}

criterion_group!(benches, fat_rlps);
criterion_main!(benches);
