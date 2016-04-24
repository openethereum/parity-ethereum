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

use util::numbers::{U256,H256};
use header::BlockNumber;

use util::bytes::{FromRawBytesVariable, FromBytesError, ToBytesWithMap};

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
	pub location: BlockLocation
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
	BranchBecomingCanonChain(BranchBecomingCanonChainData),
}

#[derive(Clone)]
pub struct BranchBecomingCanonChainData {
	/// Hash of the newest common ancestor with old canon chain.
	pub ancestor: H256,
	/// Hashes of the blocks between ancestor and this block.
	pub enacted: Vec<H256>,
	/// Hashes of the blocks which were invalidated.
	pub retracted: Vec<H256>,
}

impl FromRawBytesVariable for BranchBecomingCanonChainData {
	fn from_bytes_variable(bytes: &[u8]) -> Result<BranchBecomingCanonChainData, FromBytesError> {
		type Tuple = (Vec<H256>, Vec<H256>, H256);
		let (enacted, retracted, ancestor) = try!(Tuple::from_bytes_variable(bytes));
		Ok(BranchBecomingCanonChainData { ancestor: ancestor, enacted: enacted, retracted: retracted })
	}
}

impl FromRawBytesVariable for BlockLocation {
	fn from_bytes_variable(bytes: &[u8]) -> Result<BlockLocation, FromBytesError> {
		match bytes[0] {
			0 => Ok(BlockLocation::CanonChain),
			1 => Ok(BlockLocation::Branch),
			2 => Ok(BlockLocation::BranchBecomingCanonChain(
				try!(BranchBecomingCanonChainData::from_bytes_variable(&bytes[1..bytes.len()])))),
			_ => Err(FromBytesError::UnknownMarker)
		}
	}
}

impl ToBytesWithMap for BranchBecomingCanonChainData {
	fn to_bytes_map(&self) -> Vec<u8> {
		(&self.enacted, &self.retracted, &self.ancestor).to_bytes_map()
	}
}

impl ToBytesWithMap for BlockLocation {
	fn to_bytes_map(&self) -> Vec<u8> {
		match *self {
			BlockLocation::CanonChain => vec![0u8],
			BlockLocation::Branch => vec![1u8],
			BlockLocation::BranchBecomingCanonChain(ref data) => {
				let mut bytes = (&data.enacted, &data.retracted, &data.ancestor).to_bytes_map();
				bytes.insert(0, 2u8);
				bytes
			}
		}
	}
}
