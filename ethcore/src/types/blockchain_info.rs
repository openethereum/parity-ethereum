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

//! Blockhain info type definition

use util::numbers::*;
use header::BlockNumber;
use ipc::binary::BinaryConvertError;
use std::mem;
use std::collections::VecDeque;

/// Information about the blockchain gathered together.
#[derive(Debug, Binary)]
pub struct BlockChainInfo {
	/// Blockchain difficulty.
	pub total_difficulty: U256,
	/// Block queue difficulty.
	pub pending_total_difficulty: U256,
	/// Genesis block hash.
	pub genesis_hash: H256,
	/// Best blockchain block hash.
	pub best_block_hash: H256,
	/// Best blockchain block number.
	pub best_block_number: BlockNumber
}
