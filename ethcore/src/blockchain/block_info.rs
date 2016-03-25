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

use util::numbers::{H256, U256};
use header::BlockNumber;

/// Brief info about inserted block.
#[derive(Clone)]
pub struct BlockInfo {
	/// Block hash.
	pub hash: H256,
	/// Block number.
	pub number: BlockNumber,
	/// Total block difficulty.
	pub total_difficulty: U256,
	/// Block location in blockchain.
	pub location: BlockLocation,
}

/// Describes location of newly inserted block.
#[derive(Clone)]
pub enum BlockLocation {
	/// It's part of the canon chain.
	CanonChain,
	/// It's not a part of the canon chain.
	Branch,
	/// It's part of the fork which should become canon chain,
	/// because it's total difficulty is higher than current
	/// canon chain difficulty.
	BranchBecomingCanonChain {
		/// Hash of the newest common ancestor with old canon chain.
		ancestor: H256,
		/// Hashes of the blocks between ancestor and this block.
		enacted: Vec<H256>,
		/// Hashes of the blocks which were invalidated.
		retracted: Vec<H256>,
	},
}
