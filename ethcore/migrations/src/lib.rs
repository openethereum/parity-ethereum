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

//! Database migrations.

extern crate migration;

use migration::{ChangeColumns, SimpleMigration};

/// The migration from v10 to v11.
/// Adds a column for node info.
pub const TO_V11: ChangeColumns = ChangeColumns {
	pre_columns: Some(6),
	post_columns: Some(7),
	version: 11,
};

/// The migration from v11 to v12.
/// Adds a column for light chain storage.
pub const TO_V12: ChangeColumns = ChangeColumns {
	pre_columns: Some(7),
	post_columns: Some(8),
	version: 12,
};

#[derive(Default)]
pub struct ToV13;

impl SimpleMigration for ToV13 {
	fn columns(&self) -> Option<u32> {
		Some(8)
	}

	fn version(&self) -> u32 {
		13
	}

	fn migrated_column_index(&self) -> Option<u32> {
		// extras!
		Some(3)
	}

	fn simple_migrate(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		// remove all bloom groups
		if key[0] == 3 {
			None
		} else {
			Some((key, value))
		}
	}
}
