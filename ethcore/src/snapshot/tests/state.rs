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

use snapshot::{chunk_state, StateRebuilder};
use snapshot::io::{PackedReader, PackedWriter, SnapshotReader, SnapshotWriter};
use super::helpers::{compare_dbs, StateProducer};

use rand;
use util::hash::H256;
use util::kvdb::Database;
use util::overlaydb::{DeletionMode, OverlayDB};
use util::memorydb::MemoryDB;
use devtools::RandomTempPath;

#[test]
fn snap_and_restore() {
	let mut producer = StateProducer::new();
	let mut rng = rand::thread_rng();
	let mut old_db = MemoryDB::new();

	for _ in 0..500 {
		producer.tick(&mut rng, &mut old_db);
	}

	let snap_dir = RandomTempPath::create_dir();
	let mut snap_file = snap_dir.as_path().to_owned();
	snap_file.push("SNAP");

	let state_root = producer.state_root();
	let mut writer = PackedWriter::new(&snap_file).unwrap();

	let state_hashes = chunk_state(&old_db, &state_root, &mut writer).unwrap();

	writer.finish(::snapshot::ManifestData {
		state_hashes: state_hashes,
		block_hashes: Vec::new(),
		state_root: state_root,
		block_number: 0,
		block_hash: H256::default(),
	}).unwrap();

	let mut db_path = snap_dir.as_path().to_owned();
	db_path.push("state_db");
	{
		let new_db = Database::open_default(&db_path.to_string_lossy()).unwrap();
		let mut rebuilder = StateRebuilder::new(new_db);
		let reader = PackedReader::new(&snap_file).unwrap().unwrap();

		for chunk_hash in &reader.manifest().state_hashes {
			let raw = reader.chunk(*chunk_hash).unwrap();
			let chunk = ::util::snappy::decompress(&raw).unwrap();

			rebuilder.feed(&chunk).unwrap();
		}

		assert_eq!(rebuilder.state_root(), state_root);
	}

	let db = Database::open_default(&db_path.to_string_lossy()).unwrap();
	let new_db = OverlayDB::new(db, DeletionMode::Delete);

	compare_dbs(&old_db, &new_db);
}