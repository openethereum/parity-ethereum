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

use ethcore::client::{Client, BlockChainClient};
use common_types::ids::BlockId;
use ethereum_types::H256;

// TODO: Instead of a constant, make this based on consensus finality.
/// Number of confirmations required before request can be processed.
pub const REQUEST_CONFIRMATIONS_REQUIRED: u64 = 3;

/// Get hash of the last block with at least n confirmations.
pub fn get_confirmed_block_hash(client: &Client, confirmations: u64) -> Option<H256> {
	client.block_number(BlockId::Latest)
		.map(|b| b.saturating_sub(confirmations))
		.and_then(|b| client.block_hash(BlockId::Number(b)))
}
