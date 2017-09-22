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


//! This migration consolidates all databases into single one using Column Families.

use rlp::{Rlp, RlpStream};
use kvdb::Database;
use migration::{Batch, Config, Error, Migration, Progress};
use std::sync::Arc;

/// Which part of block to preserve
pub enum Extract {
	/// Extract block header RLP.
	Header,
	/// Extract block body RLP.
	Body,
	/// Don't change the value.
	All,
}

/// Consolidation of extras/block/state databases into single one.
pub struct ToV9 {
	progress: Progress,
	column: Option<u32>,
	extract: Extract,
}

impl ToV9 {
	/// Creates new V9 migration and assigns all `(key,value)` pairs from `source` DB to given Column Family
	pub fn new(column: Option<u32>, extract: Extract) -> Self {
		ToV9 {
			progress: Progress::default(),
			column: column,
			extract: extract,
		}
	}
}

impl Migration for ToV9 {
	fn columns(&self) -> Option<u32> { Some(5) }

	fn version(&self) -> u32 { 9 }

	fn migrate(&mut self, source: Arc<Database>, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		let mut batch = Batch::new(config, self.column);

		for (key, value) in source.iter(col).into_iter().flat_map(|inner| inner) {
			self.progress.tick();
			match self.extract {
				Extract::Header => {
					batch.insert(key.into_vec(), Rlp::new(&value).at(0).as_raw().to_vec(), dest)?
				},
				Extract::Body => {
					let mut body = RlpStream::new_list(2);
					let block_rlp = Rlp::new(&value);
					body.append_raw(block_rlp.at(1).as_raw(), 1);
					body.append_raw(block_rlp.at(2).as_raw(), 1);
					batch.insert(key.into_vec(), body.out(), dest)?
				},
				Extract::All => {
					batch.insert(key.into_vec(), value.into_vec(), dest)?
				}
			}
		}

		batch.commit(dest)
	}
}
