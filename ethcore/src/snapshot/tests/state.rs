// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! State snapshotting tests.

use snapshot::{chunk_state, Progress, StateRebuilder};
use snapshot::io::{PackedReader, PackedWriter, SnapshotReader, SnapshotWriter};
use super::helpers::{compare_dbs, StateProducer};

use rand::{XorShiftRng, SeedableRng};
use util::hash::H256;
use util::journaldb::{self, Algorithm};
use util::kvdb::{Database, DatabaseConfig};
use util::memorydb::MemoryDB;
use util::Mutex;
use devtools::RandomTempPath;

use std::sync::Arc;

#[test]
fn snap_and_restore() {
	let mut producer = StateProducer::new();
	let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);
	let mut old_db = MemoryDB::new();
	let db_cfg = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	for _ in 0..150 {
		producer.tick(&mut rng, &mut old_db);
	}

	let snap_dir = RandomTempPath::create_dir();
	let mut snap_file = snap_dir.as_path().to_owned();
	snap_file.push("SNAP");

	let state_root = producer.state_root();
	let writer = Mutex::new(PackedWriter::new(&snap_file).unwrap());

	let state_hashes = chunk_state(&old_db, &state_root, &writer, &Progress::default()).unwrap();

	writer.into_inner().finish(::snapshot::ManifestData {
		state_hashes: state_hashes,
		block_hashes: Vec::new(),
		state_root: state_root,
		block_number: 0,
		block_hash: H256::default(),
	}).unwrap();

	let mut db_path = snap_dir.as_path().to_owned();
	db_path.push("db");
	let db = {
		let new_db = Arc::new(Database::open(&db_cfg, &db_path.to_string_lossy()).unwrap());
		let mut rebuilder = StateRebuilder::new(new_db.clone(), Algorithm::Archive);
		let reader = PackedReader::new(&snap_file).unwrap().unwrap();

		for chunk_hash in &reader.manifest().state_hashes {
			let raw = reader.chunk(*chunk_hash).unwrap();
			let chunk = ::util::snappy::decompress(&raw).unwrap();

			rebuilder.feed(&chunk).unwrap();
		}

		assert_eq!(rebuilder.state_root(), state_root);
		rebuilder.check_missing().unwrap();

		new_db
	};

	let new_db = journaldb::new(db, Algorithm::Archive, ::db::COL_STATE);

	compare_dbs(&old_db, new_db.as_hashdb());
}
