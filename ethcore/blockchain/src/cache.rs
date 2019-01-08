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

/// Represents blockchain's in-memory cache size in bytes.
#[derive(Debug)]
pub struct CacheSize {
	/// Blocks cache size.
	pub blocks: usize,
	/// BlockDetails cache size.
	pub block_details: usize,
	/// Transaction addresses cache size.
	pub transaction_addresses: usize,
	/// Block receipts size.
	pub block_receipts: usize,
}

impl CacheSize {
	/// Total amount used by the cache.
	pub fn total(&self) -> usize {
		self.blocks + self.block_details + self.transaction_addresses + self.block_receipts
	}
}
