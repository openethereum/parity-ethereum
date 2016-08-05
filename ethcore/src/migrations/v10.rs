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

//! Adds an archive column family.

use util::migration::{Migration, Error, Config, Batch, Progress};
use util::journaldb::Algorithm;
use util::kvdb::Database;

const NUM_COLUMNS: Option<u32> = Some(6);
const COL_STATE: Option<u32> = Some(0);
const COL_STATE_ARCHIVE: Option<u32> = Some(5);

/// This migration adds the state archive column family.
/// In the case of pruning=archive, this will copy all entries in the
/// state column into the state archive. Other pruning methods don't use.
pub struct ToV10 {
	progress: Progress,
	pruning: Algorithm,
}

impl ToV10 {
	/// Create a new `ToV10` migration from a pruning algorithm.
	pub fn new(pruning: Algorithm) -> Self {
		ToV10 {
			progress: Progress::default(),
			pruning: pruning,
		}
	}
}

impl Migration for ToV10 {
	fn version(&self) -> u32 { 10 }
	fn columns(&self) -> Option<u32> { NUM_COLUMNS }

	fn migrate(&mut self, source: &Database, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		if col == COL_STATE_ARCHIVE { return Ok(()) }

		let mut batch = Batch::new(config, col);
		let mut archive_batch = if self.pruning == Algorithm::Archive && col == COL_STATE {
			Some(Batch::new(config, COL_STATE_ARCHIVE))
		} else {
			None
		};

		for (key, value) in source.iter(col) {
			self.progress.tick();

			if let Some(ref mut b) = archive_batch.as_mut() {
				try!(b.insert(key.clone().to_vec(), value.clone().to_vec(), dest));
			}
			try!(batch.insert(key.to_vec(), value.to_vec(), dest));
		}

		try!(batch.commit(dest));
		if let Some(mut b) = archive_batch {
			try!(b.commit(dest));
		}

		Ok(())
	}
}