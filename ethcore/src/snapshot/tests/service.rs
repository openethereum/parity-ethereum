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

//! Tests for the snapshot service.

use std::sync::Arc;

use tempdir::TempDir;
use client::{Client, BlockInfo};
use ids::BlockId;
use snapshot::service::{Service, ServiceParams};
use snapshot::{self, ManifestData, SnapshotService};
use spec::Spec;
use test_helpers::generate_dummy_client_with_spec_and_data;
use test_helpers_internal::restoration_db_handler;

use io::IoChannel;
use kvdb_rocksdb::{Database, DatabaseConfig};

struct NoopDBRestore;

impl snapshot::DatabaseRestore for NoopDBRestore {
	fn restore_db(&self, _new_db: &str) -> Result<(), ::error::Error> {
		Ok(())
	}
}

#[test]
fn restored_is_equivalent() {
	const NUM_BLOCKS: u32 = 400;
	const TX_PER: usize = 5;

	let gas_prices = vec![1.into(), 2.into(), 3.into(), 999.into()];

	let client = generate_dummy_client_with_spec_and_data(Spec::new_null, NUM_BLOCKS, TX_PER, &gas_prices);

	let tempdir = TempDir::new("").unwrap();
	let client_db = tempdir.path().join("client_db");
	let path = tempdir.path().join("snapshot");

	let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);
	let client_db = Database::open(&db_config, client_db.to_str().unwrap()).unwrap();

	let spec = Spec::new_null();
	let client2 = Client::new(
		Default::default(),
		&spec,
		Arc::new(client_db),
		Arc::new(::miner::Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();

	let service_params = ServiceParams {
		engine: spec.engine.clone(),
		genesis_block: spec.genesis_block(),
		restoration_db_handler: restoration_db_handler(db_config),
		pruning: ::journaldb::Algorithm::Archive,
		channel: IoChannel::disconnected(),
		snapshot_root: path,
		db_restore: client2.clone(),
	};

	let service = Service::new(service_params).unwrap();
	service.take_snapshot(&client, NUM_BLOCKS as u64).unwrap();

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

	assert_eq!(service.status(), ::snapshot::RestorationStatus::Inactive);

	for x in 0..NUM_BLOCKS {
		let block1 = client.block(BlockId::Number(x as u64)).unwrap();
		let block2 = client2.block(BlockId::Number(x as u64)).unwrap();

		assert_eq!(block1, block2);
	}
}

#[test]
fn guards_delete_folders() {
	let spec = Spec::new_null();
	let tempdir = TempDir::new("").unwrap();
	let service_params = ServiceParams {
		engine: spec.engine.clone(),
		genesis_block: spec.genesis_block(),
		restoration_db_handler: restoration_db_handler(DatabaseConfig::with_columns(::db::NUM_COLUMNS)),
		pruning: ::journaldb::Algorithm::Archive,
		channel: IoChannel::disconnected(),
		snapshot_root: tempdir.path().to_owned(),
		db_restore: Arc::new(NoopDBRestore),
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

	service.abort_restore();
	assert!(!path.exists());

	service.init_restore(manifest.clone(), true).unwrap();
	assert!(path.exists());

	drop(service);
	assert!(!path.exists());
}
