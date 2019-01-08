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

use ethereum_types::{H256, U256};

use common_types::{encoded, BlockNumber};
use common_types::header::Header;

/// Contains information on a best block that is specific to the consensus engine.
///
/// For GHOST fork-choice rule it would typically describe the block with highest
/// combined difficulty (usually the block with the highest block number).
///
/// Sometimes refered as 'latest block'.
pub struct BestBlock {
	/// Best block decoded header.
	pub header: Header,
	/// Best block uncompressed bytes.
	pub block: encoded::Block,
	/// Best block total difficulty.
	pub total_difficulty: U256,
}

/// Best ancient block info. If the blockchain has a gap this keeps track of where it starts.
#[derive(Default)]
pub struct BestAncientBlock {
	/// Best block hash.
	pub hash: H256,
	/// Best block number.
	pub number: BlockNumber,
}
