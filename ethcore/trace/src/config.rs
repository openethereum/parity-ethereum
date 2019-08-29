// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Traces config.

/// Traces config.
#[derive(Debug, PartialEq, Clone)]
pub struct Config {
	/// Indicates if tracing should be enabled or not.
	/// If it's None, it will be automatically configured.
	pub enabled: bool,
	/// Preferred cache-size.
	pub pref_cache_size: usize,
	/// Max cache-size.
	pub max_cache_size: usize,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			enabled: false,
			pref_cache_size: 15 * 1024 * 1024,
			max_cache_size: 20 * 1024 * 1024,
		}
	}
}
