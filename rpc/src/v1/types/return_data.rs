// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Tree route info type definition

use ethereum_types::H256;
use v1::types::Bytes;

/// Represents return data of a contract function call
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct ReturnData {
    /// Hash of the transactions.
	#[serde(rename="transactionHash")]
	pub transaction_hash: H256,
    /// Return data as bytes.
	#[serde(rename="returnData")]
	pub return_data: Bytes,
    /// Indication if the block the transaction was
    /// included on was removed due to a chain re-org
	pub removed: bool,
}
