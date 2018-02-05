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

#[macro_use]
extern crate log;
#[macro_use]
extern crate macros;
extern crate migration;
extern crate rlp;
extern crate ethereum_types;
extern crate ethcore_bytes as bytes;
extern crate kvdb;
extern crate kvdb_rocksdb;
extern crate keccak_hash as hash;
extern crate journaldb;
extern crate ethcore_bloom_journal as bloom_journal;
extern crate ethcore;
extern crate patricia_trie as trie;

use migration::ChangeColumns;

pub mod state;
pub mod blocks;
pub mod extras;

mod v9;
pub use self::v9::ToV9;
pub use self::v9::Extract;

mod v10;
pub use self::v10::ToV10;

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
