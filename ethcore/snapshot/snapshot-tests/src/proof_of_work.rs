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

//! PoW block chunker and rebuilder tests.

use std::sync::atomic::AtomicBool;
use tempdir::TempDir;
use common_types::{
	errors::{EthcoreError as Error, SnapshotError},
	engines::ForkChoice,
	snapshot::{Progress, ManifestData},
};
use blockchain::generator::{BlockGenerator, BlockBuilder};
use blockchain::{BlockChain, ExtrasInsert};
use snapshot::{
	chunk_secondary,
	SnapshotComponents,
	io::{PackedReader, PackedWriter, SnapshotReader, SnapshotWriter},
	PowSnapshot,
};
use parking_lot::{Mutex, RwLock};
use snappy;
use keccak_hash::KECCAK_NULL_RLP;
use kvdb::DBTransaction;
use ethcore::test_helpers;
use spec;

const SNAPSHOT_MODE: PowSnapshot = PowSnapshot { blocks: 30000, max_restore_blocks: 30000 };

fn chunk_and_restore(amount: u64) {
	let genesis = BlockBuilder::genesis();
	let rest = genesis.add_blocks(amount as usize);
	let generator = BlockGenerator::new(vec![rest]);
	let genesis = genesis.last();

	let engine = spec::new_test().engine;
	let tempdir = TempDir::new("").unwrap();
	let snapshot_path = tempdir.path().join("SNAP");

	let old_db = test_helpers::new_db();
	let bc = BlockChain::new(Default::default(), genesis.encoded().raw(), old_db.clone());

	// build the blockchain.
	let mut batch = DBTransaction::new();
	for block in generator {
		bc.insert_block(&mut batch, block.encoded(), vec![], ExtrasInsert {
			fork_choice: ForkChoice::New,
			is_finalized: false,
		});
		bc.commit();
	}

	old_db.key_value().write(batch).unwrap();

	let best_hash = bc.best_block_hash();

	// snapshot it.
	let writer = Mutex::new(PackedWriter::new(&snapshot_path).unwrap());
	let block_hashes = chunk_secondary(
		Box::new(SNAPSHOT_MODE),
		&bc,
		best_hash,
		&writer,
		&RwLock::new(Progress::new())
	).unwrap();

	let manifest = ManifestData {
		version: 2,
		state_hashes: Vec::new(),
		block_hashes,
		state_root: KECCAK_NULL_RLP,
		block_number: amount,
		block_hash: best_hash,
	};

	writer.into_inner().finish(manifest.clone()).unwrap();

	// restore it.
	let new_db = test_helpers::new_db();
	let new_chain = BlockChain::new(Default::default(), genesis.encoded().raw(), new_db.clone());
	let mut rebuilder = SNAPSHOT_MODE.rebuilder(new_chain, new_db.clone(), &manifest).unwrap();

	let reader = PackedReader::new(&snapshot_path).unwrap().unwrap();
	let flag = AtomicBool::new(true);
	for chunk_hash in &reader.manifest().block_hashes {
		let compressed = reader.chunk(*chunk_hash).unwrap();
		let chunk = snappy::decompress(&compressed).unwrap();
		rebuilder.feed(&chunk, engine.as_ref(), &flag).unwrap();
	}

	rebuilder.finalize().unwrap();
	drop(rebuilder);

	// and test it.
	let new_chain = BlockChain::new(Default::default(), genesis.encoded().raw(), new_db);
	assert_eq!(new_chain.best_block_hash(), best_hash);
}

#[test]
fn chunk_and_restore_500() {
	chunk_and_restore(500)
}

#[test]
fn chunk_and_restore_4k() {
	chunk_and_restore(4000)
}

#[test]
fn checks_flag() {
	use rlp::RlpStream;
	use ethereum_types::H256;

	let mut stream = RlpStream::new_list(5);

	stream.append(&100u64)
		.append(&H256::zero())
		.append(&(!0u64));

	stream.append_empty_data().append_empty_data();

	let genesis = BlockBuilder::genesis();
	let chunk = stream.out();

	let db = test_helpers::new_db();
	let engine = spec::new_test().engine;
	let chain = BlockChain::new(Default::default(), genesis.last().encoded().raw(), db.clone());

	let manifest = ManifestData {
		version: 2,
		state_hashes: Vec::new(),
		block_hashes: Vec::new(),
		state_root: KECCAK_NULL_RLP,
		block_number: 102,
		block_hash: H256::zero(),
	};

	let mut rebuilder = SNAPSHOT_MODE.rebuilder(chain, db.clone(), &manifest).unwrap();

	match rebuilder.feed(&chunk, engine.as_ref(), &AtomicBool::new(false)) {
		Err(Error::Snapshot(SnapshotError::RestorationAborted)) => {}
		_ => panic!("Wrong result on abort flag set")
	}
}
