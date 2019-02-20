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

//! State snapshotting tests.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use hash::{KECCAK_NULL_RLP, keccak};

use types::basic_account::BasicAccount;
use snapshot::account;
use snapshot::{chunk_state, Error as SnapshotError, Progress, StateRebuilder, SNAPSHOT_SUBPARTS};
use snapshot::io::{PackedReader, PackedWriter, SnapshotReader, SnapshotWriter};
use super::helpers::StateProducer;

use error::{Error, ErrorKind};

use rand::{XorShiftRng, SeedableRng};
use ethereum_types::H256;
use journaldb::{self, Algorithm};
use kvdb_rocksdb::{Database, DatabaseConfig};
use parking_lot::Mutex;
use tempdir::TempDir;

#[test]
fn snap_and_restore() {
	use hash_db::HashDB;
	let mut producer = StateProducer::new();
	let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);
	let mut old_db = journaldb::new_memory_db();
	let db_cfg = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	for _ in 0..150 {
		producer.tick(&mut rng, &mut old_db);
	}

	let tempdir = TempDir::new("").unwrap();
	let snap_file = tempdir.path().join("SNAP");

	let state_root = producer.state_root();
	let writer = Mutex::new(PackedWriter::new(&snap_file).unwrap());

	let mut state_hashes = Vec::new();
	for part in 0..SNAPSHOT_SUBPARTS {
		let mut hashes = chunk_state(&old_db, &state_root, &writer, &Progress::default(), Some(part)).unwrap();
		state_hashes.append(&mut hashes);
	}

	writer.into_inner().finish(::snapshot::ManifestData {
		version: 2,
		state_hashes: state_hashes,
		block_hashes: Vec::new(),
		state_root: state_root,
		block_number: 1000,
		block_hash: H256::default(),
	}).unwrap();

	let db_path = tempdir.path().join("db");
	let db = {
		let new_db = Arc::new(Database::open(&db_cfg, &db_path.to_string_lossy()).unwrap());
		let mut rebuilder = StateRebuilder::new(new_db.clone(), Algorithm::OverlayRecent);
		let reader = PackedReader::new(&snap_file).unwrap().unwrap();

		let flag = AtomicBool::new(true);

		for chunk_hash in &reader.manifest().state_hashes {
			let raw = reader.chunk(*chunk_hash).unwrap();
			let chunk = ::snappy::decompress(&raw).unwrap();

			rebuilder.feed(&chunk, &flag).unwrap();
		}

		assert_eq!(rebuilder.state_root(), state_root);
		rebuilder.finalize(1000, H256::default()).unwrap();

		new_db
	};

	let new_db = journaldb::new(db, Algorithm::OverlayRecent, ::db::COL_STATE);
	assert_eq!(new_db.earliest_era(), Some(1000));
	let keys = old_db.keys();

	for key in keys.keys() {
		assert_eq!(old_db.get(&key).unwrap(), new_db.as_hash_db().get(&key).unwrap());
	}
}

#[test]
fn get_code_from_prev_chunk() {
	use std::collections::HashSet;
	use rlp::RlpStream;
	use ethereum_types::{H256, U256};
	use hash_db::HashDB;

	use account_db::{AccountDBMut, AccountDB};

	let code = b"this is definitely code";
	let mut used_code = HashSet::new();
	let mut acc_stream = RlpStream::new_list(4);
	acc_stream.append(&U256::default())
		.append(&U256::default())
		.append(&KECCAK_NULL_RLP)
		.append(&keccak(code));

	let (h1, h2) = (H256::random(), H256::random());

	// two accounts with the same code, one per chunk.
	// first one will have code inlined,
	// second will just have its hash.
	let thin_rlp = acc_stream.out();
	let acc: BasicAccount = ::rlp::decode(&thin_rlp).expect("error decoding basic account");

	let mut make_chunk = |acc, hash| {
		let mut db = journaldb::new_memory_db();
		AccountDBMut::from_hash(&mut db, hash).insert(&code[..]);

		let fat_rlp = account::to_fat_rlps(&hash, &acc, &AccountDB::from_hash(&db, hash), &mut used_code, usize::max_value(), usize::max_value()).unwrap();
		let mut stream = RlpStream::new_list(1);
		stream.append_raw(&fat_rlp[0], 1);
		stream.out()
	};

	let chunk1 = make_chunk(acc.clone(), h1);
	let chunk2 = make_chunk(acc, h2);

	let tempdir = TempDir::new("").unwrap();
	let db_cfg = DatabaseConfig::with_columns(::db::NUM_COLUMNS);
	let new_db = Arc::new(Database::open(&db_cfg, tempdir.path().to_str().unwrap()).unwrap());

	{
		let mut rebuilder = StateRebuilder::new(new_db.clone(), Algorithm::OverlayRecent);
		let flag = AtomicBool::new(true);

		rebuilder.feed(&chunk1, &flag).unwrap();
		rebuilder.feed(&chunk2, &flag).unwrap();

		rebuilder.finalize(1000, H256::random()).unwrap();
	}

	let state_db = journaldb::new(new_db, Algorithm::OverlayRecent, ::db::COL_STATE);
	assert_eq!(state_db.earliest_era(), Some(1000));
}

#[test]
fn checks_flag() {
	let mut producer = StateProducer::new();
	let mut rng = XorShiftRng::from_seed([5, 6, 7, 8]);
	let mut old_db = journaldb::new_memory_db();
	let db_cfg = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	for _ in 0..10 {
		producer.tick(&mut rng, &mut old_db);
	}

	let tempdir = TempDir::new("").unwrap();
	let snap_file = tempdir.path().join("SNAP");

	let state_root = producer.state_root();
	let writer = Mutex::new(PackedWriter::new(&snap_file).unwrap());

	let state_hashes = chunk_state(&old_db, &state_root, &writer, &Progress::default(), None).unwrap();

	writer.into_inner().finish(::snapshot::ManifestData {
		version: 2,
		state_hashes: state_hashes,
		block_hashes: Vec::new(),
		state_root: state_root,
		block_number: 0,
		block_hash: H256::default(),
	}).unwrap();

	let tempdir = TempDir::new("").unwrap();
	let db_path = tempdir.path().join("db");
	{
		let new_db = Arc::new(Database::open(&db_cfg, &db_path.to_string_lossy()).unwrap());
		let mut rebuilder = StateRebuilder::new(new_db.clone(), Algorithm::OverlayRecent);
		let reader = PackedReader::new(&snap_file).unwrap().unwrap();

		let flag = AtomicBool::new(false);

		for chunk_hash in &reader.manifest().state_hashes {
			let raw = reader.chunk(*chunk_hash).unwrap();
			let chunk = ::snappy::decompress(&raw).unwrap();

			match rebuilder.feed(&chunk, &flag) {
				Err(Error(ErrorKind::Snapshot(SnapshotError::RestorationAborted), _)) => {},
				_ => panic!("unexpected result when feeding with flag off"),
			}
		}
	}
}
