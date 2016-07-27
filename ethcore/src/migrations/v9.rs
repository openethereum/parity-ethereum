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


//! This migration consolidates all databases into single one using Column Families.

use util::kvdb::Database;
use util::migration::{Batch, Config, Error, Migration, Progress};

/// Consolidation of extras/block/state databases into single one.
pub struct ToV9 {
	progress: Progress,
	column: Option<u32>,
}

impl ToV9 {
	/// Creates new V9 migration and assigns all `(key,value)` pairs from `source` DB to given Column Family
	pub fn new(column: Option<u32>) -> Self {
		ToV9 {
			progress: Progress::default(),
			column: column,
		}
	}
}

impl Migration for ToV9 {

	fn columns(&self) -> Option<u32> { Some(5) }

	fn version(&self) -> u32 { 9 }

	fn migrate(&mut self, source: &Database, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		let mut batch = Batch::new(config, self.column);

		for (key, value) in source.iter(col) {
			self.progress.tick();
			try!(batch.insert(key.to_vec(), value.to_vec(), dest));
		}

		batch.commit(dest)
	}
}
