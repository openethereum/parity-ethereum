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

use ethereum_types::{H256, U256, Address};
use ethjson;
use hash::KECCAK_NULL_RLP;
use spec::seal::Seal;

/// Genesis components.
pub struct Genesis {
	/// Seal.
	pub seal: Seal,
	/// Difficulty.
	pub difficulty: U256,
	/// Author.
	pub author: Address,
	/// Timestamp.
	pub timestamp: u64,
	/// Parent hash.
	pub parent_hash: H256,
	/// Gas limit.
	pub gas_limit: U256,
	/// Transactions root.
	pub transactions_root: H256,
	/// Receipts root.
	pub receipts_root: H256,
	/// State root.
	pub state_root: Option<H256>,
	/// Gas used.
	pub gas_used: U256,
	/// Extra data.
	pub extra_data: Vec<u8>,
}

impl From<ethjson::spec::Genesis> for Genesis {
	fn from(g: ethjson::spec::Genesis) -> Self {
		Genesis {
			seal: From::from(g.seal),
			difficulty: g.difficulty.into(),
			author: g.author.map_or_else(Address::zero, Into::into),
			timestamp: g.timestamp.map_or(0, Into::into),
			parent_hash: g.parent_hash.map_or_else(H256::zero, Into::into),
			gas_limit: g.gas_limit.into(),
			transactions_root: g.transactions_root.map_or_else(|| KECCAK_NULL_RLP.clone(), Into::into),
			receipts_root: g.receipts_root.map_or_else(|| KECCAK_NULL_RLP.clone(), Into::into),
			state_root: g.state_root.map(Into::into),
			gas_used: g.gas_used.map_or_else(U256::zero, Into::into),
			extra_data: g.extra_data.map_or_else(Vec::new, Into::into),
		}
	}
}
