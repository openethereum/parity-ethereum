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

/// Configuration for application cache sizes.
/// All	values are represented in MB.
#[derive(Debug, PartialEq)]
pub struct CacheConfig {
	/// Size of rocksdb cache set using option `set_block_cache_size_mb`
	/// 50% is blockchain
	/// 25% is tracing
	/// 25% is state
	pub rocksdb: u32,
	/// Size of blockchain cache.
	pub blockchain: u32,
	/// Size of transaction queue cache.
	pub queue: u32,
}

impl Default for CacheConfig {
	fn default() -> Self {
		CacheConfig::new_with_total_cache_size(200)
	}
}

impl CacheConfig {
	/// Creates new cache config with cumulative size equal `total`.
	pub fn new_with_total_cache_size(total: u32) -> Self {
		CacheConfig {
			rocksdb: total / 2,
			blockchain: total / 4,
			queue: total / 4,
		}
	}

	/// Size of rocksdb cache for blockchain.
	pub fn rocksdb_blockchain_cache_size(&self) -> u32 {
		self.rocksdb / 2
	}

	/// Size of rocksdb cache for traces.
	pub fn rocksdb_tracing_cache_size(&self) -> u32 {
		self.rocksdb / 4
	}

	/// Size of rocksdb cache for state.
	pub fn rocksdb_state_cache_size(&self) -> u32 {
		self.rocksdb / 4
	}
}

#[cfg(test)]
mod tests {
	use super::CacheConfig;

	#[test]
	fn test_cache_config_constructor() {
		let config = CacheConfig::new_with_total_cache_size(200);
		assert_eq!(config.rocksdb, 100);
		assert_eq!(config.blockchain, 50);
		assert_eq!(config.queue, 50);
	}

	#[test]
	fn test_cache_config_rocksdb_cache_sizes() {
		let config = CacheConfig::new_with_total_cache_size(400);
		assert_eq!(config.rocksdb, 200);
		assert_eq!(config.rocksdb_blockchain_cache_size(), 100);
		assert_eq!(config.rocksdb_tracing_cache_size(), 50);
		assert_eq!(config.rocksdb_state_cache_size(), 50);
	}

	#[test]
	fn test_cache_config_default() {
		assert_eq!(CacheConfig::default(), CacheConfig::new_with_total_cache_size(200));
	}
}
