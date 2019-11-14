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

//! Tests for the snapshot service.

use std::fs;
use std::sync::Arc;

use tempdir::TempDir;
use blockchain::BlockProvider;
use ethcore::client::{Client, ClientConfig};
use client_traits::{BlockInfo, ImportBlock};
use common_types::{
	io_message::ClientIoMessage,
	ids::BlockId,
	snapshot::Progress,
	verification::Unverified,
	snapshot::{ManifestData, RestorationStatus},
};
use snapshot::{
	chunk_state, chunk_secondary, SnapshotService,
	io::{PackedReader, PackedWriter, SnapshotReader, SnapshotWriter},
	service::{Service, ServiceParams, Guard, Restoration, RestorationParams},
	PowSnapshot,
};
use spec;
use ethcore::{
	miner,
	test_helpers::{new_db, new_temp_db, generate_dummy_client_with_spec_and_data, restoration_db_handler}
};

use parking_lot::{Mutex, RwLock};
use ethcore_io::{IoChannel, IoService};
use kvdb_rocksdb::DatabaseConfig;
use journaldb::Algorithm;

#[test]
fn sends_async_messages() {
	let gas_prices = vec![1.into(), 2.into(), 3.into(), 999.into()];
	let client = generate_dummy_client_with_spec_and_data(spec::new_null, 400, 5, &gas_prices);
	let service = IoService::<ClientIoMessage<Client>>::start().unwrap();
	let spec = spec::new_test();

	let tempdir = TempDir::new("").unwrap();
	let dir = tempdir.path().join("snapshot");

	let snapshot_params = ServiceParams {
		engine: spec.engine.clone(),
		genesis_block: spec.genesis_block(),
		restoration_db_handler: restoration_db_handler(Default::default()),
		pruning: Algorithm::Archive,
		channel: service.channel(),
		snapshot_root: dir,
		client,
	};

	let service = Service::new(snapshot_params).unwrap();

	assert!(service.manifest().is_none());
	assert!(service.chunk(Default::default()).is_none());
	assert_eq!(service.status(), RestorationStatus::Inactive);

	let manifest = ManifestData {
		version: 2,
		state_hashes: vec![],
		block_hashes: vec![],
		state_root: Default::default(),
		block_number: 0,
		block_hash: Default::default(),
	};

	service.begin_restore(manifest);
	service.abort_restore();
	service.restore_state_chunk(Default::default(), vec![]);
	service.restore_block_chunk(Default::default(), vec![]);
}

#[test]
fn cannot_finish_with_invalid_chunks() {
	use ethereum_types::H256;
	use kvdb_rocksdb::DatabaseConfig;

	let spec = spec::new_test();
	let tempdir = TempDir::new("").unwrap();

	let state_hashes: Vec<_> = (0..5).map(|_| H256::random()).collect();
	let block_hashes: Vec<_> = (0..5).map(|_| H256::random()).collect();
	let db_config = DatabaseConfig::with_columns(ethcore_db::NUM_COLUMNS);
	let gb = spec.genesis_block();
	let flag = ::std::sync::atomic::AtomicBool::new(true);

	let engine = &*spec.engine.clone();
	let params = RestorationParams::new(
		ManifestData {
			version: 2,
			state_hashes: state_hashes.clone(),
			block_hashes: block_hashes.clone(),
			state_root: H256::zero(),
			block_number: 100000,
			block_hash: H256::zero(),
		},
		Algorithm::Archive,
		restoration_db_handler(db_config).open(&tempdir.path().to_owned()).unwrap(),
		None,
		&gb,
		Guard::benign(),
		engine,
	);

	let mut restoration = Restoration::new(params).unwrap();
	let definitely_bad_chunk = [1, 2, 3, 4, 5];

	for hash in state_hashes {
		assert!(restoration.feed_state(hash, &definitely_bad_chunk, &flag).is_err());
		assert!(!restoration.is_done());
	}

	for hash in block_hashes {
		assert!(restoration.feed_blocks(hash, &definitely_bad_chunk, &*spec.engine, &flag).is_err());
		assert!(!restoration.is_done());
	}
}


#[test]
fn restored_is_equivalent() {
	let _ = ::env_logger::try_init();

	const NUM_BLOCKS: u32 = 400;
	const TX_PER: usize = 5;

	let gas_prices = vec![1.into(), 2.into(), 3.into(), 999.into()];
	let client = generate_dummy_client_with_spec_and_data(spec::new_null, NUM_BLOCKS, TX_PER, &gas_prices);

	let tempdir = TempDir::new("").unwrap();
	let client_db = tempdir.path().join("client_db");
	let path = tempdir.path().join("snapshot");

	let db_config = DatabaseConfig::with_columns(ethcore_db::NUM_COLUMNS);
	let restoration = restoration_db_handler(db_config);
	let blockchain_db = restoration.open(&client_db).unwrap();

	let spec = spec::new_null();
	let client2 = Client::new(
		Default::default(),
		&spec,
		blockchain_db,
		Arc::new(miner::Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();

	let service_params = ServiceParams {
		engine: spec.engine.clone(),
		genesis_block: spec.genesis_block(),
		restoration_db_handler: restoration,
		pruning: ::journaldb::Algorithm::Archive,
		channel: IoChannel::disconnected(),
		snapshot_root: path,
		client: client2.clone(),
	};

	let service = Service::new(service_params).unwrap();
	service.take_snapshot(&*client, NUM_BLOCKS as u64).unwrap();

	let manifest = service.manifest().unwrap();

	service.init_restore(manifest.clone(), true).unwrap();
	assert!(service.init_restore(manifest.clone(), true).is_ok());

	for hash in manifest.state_hashes {
		let chunk = service.chunk(hash).unwrap();
		service.feed_state_chunk(hash, &chunk);
	}

	for hash in manifest.block_hashes {
		let chunk = service.chunk(hash).unwrap();
		service.feed_block_chunk(hash, &chunk);
	}

	assert_eq!(service.status(), RestorationStatus::Inactive);

	for x in 0..NUM_BLOCKS {
		let block1 = client.block(BlockId::Number(x as u64)).unwrap();
		let block2 = client2.block(BlockId::Number(x as u64)).unwrap();

		assert_eq!(block1, block2);
	}
}

// on windows the guards deletion (remove_dir_all)
// is not happening (error directory is not empty).
// So the test is disabled until windows api behave.
#[cfg(not(target_os = "windows"))]
#[test]
fn guards_delete_folders() {
	let gas_prices = vec![1.into(), 2.into(), 3.into(), 999.into()];
	let client = generate_dummy_client_with_spec_and_data(spec::new_null, 400, 5, &gas_prices);

	let spec = spec::new_null();
	let tempdir = TempDir::new("").unwrap();
	let service_params = ServiceParams {
		engine: spec.engine.clone(),
		genesis_block: spec.genesis_block(),
		restoration_db_handler: restoration_db_handler(DatabaseConfig::with_columns(ethcore_db::NUM_COLUMNS)),
		pruning: ::journaldb::Algorithm::Archive,
		channel: IoChannel::disconnected(),
		snapshot_root: tempdir.path().to_owned(),
		client: client,
	};

	let service = Service::new(service_params).unwrap();
	let path = tempdir.path().join("restoration");

	let manifest = ManifestData {
		version: 2,
		state_hashes: vec![],
		block_hashes: vec![],
		block_number: 0,
		block_hash: Default::default(),
		state_root: Default::default(),
	};

	service.init_restore(manifest.clone(), true).unwrap();
	assert!(path.exists());

	// The `db` folder should have been deleted,
	// while the `temp` one kept
	service.abort_restore();
	assert!(!path.join("db").exists());
	assert!(path.join("temp").exists());

	service.init_restore(manifest.clone(), true).unwrap();
	assert!(path.exists());

	drop(service);
	assert!(!path.join("db").exists());
	assert!(path.join("temp").exists());
}

#[test]
fn keep_ancient_blocks() {
	let _ = ::env_logger::try_init();
	// Test variables
	const NUM_BLOCKS: u64 = 500;
	const NUM_SNAPSHOT_BLOCKS: u64 = 300;
	const SNAPSHOT_MODE: PowSnapshot = PowSnapshot { blocks: NUM_SNAPSHOT_BLOCKS, max_restore_blocks: NUM_SNAPSHOT_BLOCKS };

	// Temporary folders
	let tempdir = TempDir::new("").unwrap();
	let snapshot_path = tempdir.path().join("SNAP");

	// Generate blocks
	let gas_prices = vec![1.into(), 2.into(), 3.into(), 999.into()];
	let spec_f = spec::new_null;
	let spec = spec_f();
	let client = generate_dummy_client_with_spec_and_data(spec_f, NUM_BLOCKS as u32, 5, &gas_prices);

	let bc = client.chain();

	// Create the Snapshot
	let best_hash = bc.best_block_hash();
	let writer = Mutex::new(PackedWriter::new(&snapshot_path).unwrap());
	let block_hashes = chunk_secondary(
		Box::new(SNAPSHOT_MODE),
		&bc,
		best_hash,
		&writer,
		&RwLock::new(Progress::new())
	).unwrap();
	let state_db = client.state_db().journal_db().boxed_clone();
	let start_header = bc.block_header_data(&best_hash).unwrap();
	let state_root = start_header.state_root();
	let state_hashes = chunk_state(
		state_db.as_hash_db(),
		&state_root,
		&writer,
		&RwLock::new(Progress::new()),
		None,
		0
	).unwrap();

	let manifest = ManifestData {
		version: 2,
		state_hashes,
		state_root,
		block_hashes,
		block_number: NUM_BLOCKS,
		block_hash: best_hash,
	};

	writer.into_inner().finish(manifest.clone()).unwrap();

	// Initialize the Client
	let db_config = DatabaseConfig::with_columns(ethcore_db::NUM_COLUMNS);
	let client_db = new_temp_db(&tempdir.path());
	let client2 = Client::new(
		ClientConfig::default(),
		&spec,
		client_db,
		Arc::new(miner::Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();

	// Add some ancient blocks
	for block_number in 1..50 {
		let block_hash = bc.block_hash(block_number).unwrap();
		let block = bc.block(&block_hash).unwrap();
		client2.import_block(Unverified::from_rlp(block.into_inner()).unwrap()).unwrap();
	}

	client2.flush_queue();

	// Restore the Snapshot
	let reader = PackedReader::new(&snapshot_path).unwrap().unwrap();
	let service_params = ServiceParams {
		engine: spec.engine.clone(),
		genesis_block: spec.genesis_block(),
		restoration_db_handler: restoration_db_handler(db_config),
		pruning: ::journaldb::Algorithm::Archive,
		channel: IoChannel::disconnected(),
		snapshot_root: tempdir.path().to_owned(),
		client: client2.clone(),
	};
	let service = Service::new(service_params).unwrap();
	service.init_restore(manifest.clone(), false).unwrap();

	for hash in &manifest.block_hashes {
		let chunk = reader.chunk(*hash).unwrap();
		service.feed_block_chunk(*hash, &chunk);
	}

	for hash in &manifest.state_hashes {
		let chunk = reader.chunk(*hash).unwrap();
		service.feed_state_chunk(*hash, &chunk);
	}

	match service.status() {
		RestorationStatus::Inactive => (),
		RestorationStatus::Failed => panic!("Snapshot Restoration has failed."),
		RestorationStatus::Ongoing { .. } => panic!("Snapshot Restoration should be done."),
		_ => panic!("Invalid Snapshot Service status."),
	}

	// Check that the latest block number is the right one
	assert_eq!(client2.block(BlockId::Latest).unwrap().number(), NUM_BLOCKS as u64);

	// Check that we have blocks in [NUM_BLOCKS - NUM_SNAPSHOT_BLOCKS + 1 ; NUM_BLOCKS]
	// but none before
	assert!(client2.block(BlockId::Number(NUM_BLOCKS - NUM_SNAPSHOT_BLOCKS + 1)).is_some());
	assert!(client2.block(BlockId::Number(100)).is_none());

	// Check that the first 50 blocks have been migrated
	for block_number in 1..49 {
		assert!(client2.block(BlockId::Number(block_number)).is_some());
	}
}

#[test]
fn recover_aborted_recovery() {
	let _ = env_logger::try_init();

	const NUM_BLOCKS: u32 = 400;
	let gas_prices = vec![1.into(), 2.into(), 3.into(), 999.into()];
	let client = generate_dummy_client_with_spec_and_data(spec::new_null, NUM_BLOCKS, 5, &gas_prices);

	let spec = spec::new_null();
	let tempdir = TempDir::new("").unwrap();
	let db_config = DatabaseConfig::with_columns(ethcore_db::NUM_COLUMNS);
	let client_db = new_db();
	let client2 = Client::new(
		Default::default(),
		&spec,
		client_db,
		Arc::new(miner::Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();
	let service_params = ServiceParams {
		engine: spec.engine.clone(),
		genesis_block: spec.genesis_block(),
		restoration_db_handler: restoration_db_handler(db_config),
		pruning: ::journaldb::Algorithm::Archive,
		channel: IoChannel::disconnected(),
		snapshot_root: tempdir.path().to_owned(),
		client: client2.clone(),
	};

	let service = Service::new(service_params).unwrap();
	service.take_snapshot(&*client, NUM_BLOCKS as u64).unwrap();

	let manifest = service.manifest().unwrap();
	service.init_restore(manifest.clone(), true).unwrap();

	// Restore only the state chunks
	for hash in &manifest.state_hashes {
		let chunk = service.chunk(*hash).unwrap();
		service.feed_state_chunk(*hash, &chunk);
	}

	match service.status() {
		RestorationStatus::Ongoing { block_chunks_done, state_chunks_done, .. } => {
			assert_eq!(state_chunks_done, manifest.state_hashes.len() as u32);
			assert_eq!(block_chunks_done, 0);
		},
		e => panic!("Snapshot restoration must be ongoing ; {:?}", e),
	}

	// Abort the restore...
	service.abort_restore();

	// And try again!
	service.init_restore(manifest.clone(), true).unwrap();

	match service.status() {
		RestorationStatus::Ongoing { block_chunks_done, state_chunks_done, .. } => {
			assert_eq!(state_chunks_done, manifest.state_hashes.len() as u32);
			assert_eq!(block_chunks_done, 0);
		},
		e => panic!("Snapshot restoration must be ongoing ; {:?}", e),
	}

	// Remove the snapshot directory, and restart the restoration
	// It shouldn't have restored any previous blocks
	fs::remove_dir_all(tempdir.path()).unwrap();

	// And try again!
	service.init_restore(manifest.clone(), true).unwrap();

	match service.status() {
		RestorationStatus::Ongoing { block_chunks_done, state_chunks_done, .. } => {
			assert_eq!(block_chunks_done, 0);
			assert_eq!(state_chunks_done, 0);
		},
		_ => panic!("Snapshot restoration must be ongoing"),
	}
}
