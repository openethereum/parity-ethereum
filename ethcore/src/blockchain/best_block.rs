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

use util::{Bytes, U256, H256};
use header::BlockNumber;

/// Best block info.
#[derive(Default)]
pub struct BestBlock {
	/// Best block hash.
	pub hash: H256,
	/// Best block number.
	pub number: BlockNumber,
	/// Best block timestamp.
	pub timestamp: u64,
	/// Best block total difficulty.
	pub total_difficulty: U256,
	/// Best block uncompressed bytes
	pub block: Bytes,
}

/// Best ancient block info. If the blockchain has a gap this keeps track of where it starts.
#[derive(Default)]
pub struct BestAncientBlock {
	/// Best block hash.
	pub hash: H256,
	/// Best block number.
	pub number: BlockNumber,
}
