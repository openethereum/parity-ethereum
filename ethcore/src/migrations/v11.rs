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


//! Adds a seventh column for node information.

use util::kvdb::Database;
use util::migration::{Batch, Config, Error, Migration, Progress};
use std::sync::Arc;

/// Copies over data for all existing columns.
#[derive(Default)]
pub struct ToV11(Progress);


impl Migration for ToV11 {
	fn pre_columns(&self) -> Option<u32> { Some(6) }
	fn columns(&self) -> Option<u32> { Some(7) }

	fn version(&self) -> u32 { 11 }

	fn migrate(&mut self, source: Arc<Database>, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		// just copy everything over.
		let mut batch = Batch::new(config, col);

		for (key, value) in source.iter(col) {
			self.0.tick();
			batch.insert(key.to_vec(), value.to_vec(), dest)?
		}

		batch.commit(dest)
	}
}
