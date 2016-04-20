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

//! Import route.

use util::hash::H256;
use blockchain::block_info::{BlockInfo, BlockLocation};

/// Import route for newly inserted block.
#[derive(Debug, PartialEq)]
pub struct ImportRoute {
	/// Blocks that were invalidated by new block.
	pub retracted: Vec<H256>,
	/// Blocks that were validated by new block.
	pub enacted: Vec<H256>,
}

impl ImportRoute {
	pub fn none() -> Self {
		ImportRoute {
			retracted: vec![],
			enacted: vec![],
		}
	}
}

impl From<BlockInfo> for ImportRoute {
	fn from(info: BlockInfo) -> ImportRoute {
		match info.location {
			BlockLocation::CanonChain => ImportRoute {
				retracted: vec![],
				enacted: vec![info.hash],
			},
			BlockLocation::Branch => ImportRoute::none(),
			BlockLocation::BranchBecomingCanonChain(mut data) => {
				data.enacted.push(info.hash);
				ImportRoute {
					retracted: data.retracted,
					enacted: data.enacted,
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use util::hash::H256;
	use util::numbers::U256;
	use blockchain::block_info::{BlockInfo, BlockLocation, BranchBecomingCanonChainData};
	use blockchain::ImportRoute;

	#[test]
	fn import_route_none() {
		assert_eq!(ImportRoute::none(), ImportRoute {
			enacted: vec![],
			retracted: vec![],
		});
	}

	#[test]
	fn import_route_branch() {
		let info = BlockInfo {
			hash: H256::from(U256::from(1)),
			number: 0,
			total_difficulty: U256::from(0),
			location: BlockLocation::Branch,
		};

		assert_eq!(ImportRoute::from(info), ImportRoute::none());
	}

	#[test]
	fn import_route_canon_chain() {
		let info = BlockInfo {
			hash: H256::from(U256::from(1)),
			number: 0,
			total_difficulty: U256::from(0),
			location: BlockLocation::CanonChain,
		};

		assert_eq!(ImportRoute::from(info), ImportRoute {
			retracted: vec![],
			enacted: vec![H256::from(U256::from(1))],
		});
	}

	#[test]
	fn import_route_branch_becoming_canon_chain() {
		let info = BlockInfo {
			hash: H256::from(U256::from(2)),
			number: 0,
			total_difficulty: U256::from(0),
			location: BlockLocation::BranchBecomingCanonChain(BranchBecomingCanonChainData {
				ancestor: H256::from(U256::from(0)),
				enacted: vec![H256::from(U256::from(1))],
				retracted: vec![H256::from(U256::from(3)), H256::from(U256::from(4))],
			})
		};

		assert_eq!(ImportRoute::from(info), ImportRoute {
			retracted: vec![H256::from(U256::from(3)), H256::from(U256::from(4))],
			enacted: vec![H256::from(U256::from(1)), H256::from(U256::from(2))],
		});
	}
}
