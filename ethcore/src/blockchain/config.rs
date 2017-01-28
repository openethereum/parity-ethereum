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

//! Blockchain configuration.

/// Blockchain configuration.
#[derive(Debug, PartialEq, Clone)]
pub struct Config {
	/// Preferred cache size in bytes.
	pub pref_cache_size: usize,
	/// Maximum cache size in bytes.
	pub max_cache_size: usize,
	/// Backing db cache_size
	pub db_cache_size: Option<usize>,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			pref_cache_size: 1 << 14,
			max_cache_size: 1 << 20,
			db_cache_size: None,
		}
	}
}

