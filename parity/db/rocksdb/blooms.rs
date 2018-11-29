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

//! Blooms migration from rocksdb to blooms-db

use std::path::Path;
use ethereum_types::Bloom;
use ethcore::error::Error;
use rlp;
use super::kvdb_rocksdb::DatabaseConfig;
use super::open_database;

const LOG_BLOOMS_ELEMENTS_PER_INDEX: u64 = 16;

pub fn migrate_blooms<P: AsRef<Path>>(path: P, config: &DatabaseConfig) -> Result<(), Error> {
	// init
	let db = open_database(&path.as_ref().to_string_lossy(), config)?;

	// possible optimization:
	// pre-allocate space on disk for faster migration

	// iterate over header blooms and insert them in blooms-db
	// Some(3) -> COL_EXTRA
	// 3u8 -> ExtrasIndex::BlocksBlooms
	// 0u8 -> level 0
	let blooms_iterator = db.key_value()
		.iter_from_prefix(Some(3), &[3u8, 0u8])
		.filter(|(key, _)| key.len() == 6)
		.take_while(|(key, _)| {
			key[0] == 3u8 && key[1] == 0u8
		})
		.map(|(key, group)| {
			let index =
				(key[2] as u64) << 24 |
				(key[3] as u64) << 16 |
				(key[4] as u64) << 8 |
				(key[5] as u64);
			let number = index * LOG_BLOOMS_ELEMENTS_PER_INDEX;

			let blooms = rlp::decode_list::<Bloom>(&group);
			(number, blooms)
		});

	for (number, blooms) in blooms_iterator {
		db.blooms().insert_blooms(number, blooms.iter())?;
	}

	// iterate over trace blooms and insert them in blooms-db
	// Some(4) -> COL_TRACE
	// 1u8 -> TraceDBIndex::BloomGroups
	// 0u8 -> level 0
	let trace_blooms_iterator = db.key_value()
		.iter_from_prefix(Some(4), &[1u8, 0u8])
		.filter(|(key, _)| key.len() == 6)
		.take_while(|(key, _)| {
			key[0] == 1u8 && key[1] == 0u8
		})
		.map(|(key, group)| {
			let index =
				(key[2] as u64) |
				(key[3] as u64) << 8 |
				(key[4] as u64) << 16 |
				(key[5] as u64) << 24;
			let number = index * LOG_BLOOMS_ELEMENTS_PER_INDEX;

			let blooms = rlp::decode_list::<Bloom>(&group);
			(number, blooms)
		});

	for (number, blooms) in trace_blooms_iterator {
		db.trace_blooms().insert_blooms(number, blooms.iter())?;
	}

	Ok(())
}
