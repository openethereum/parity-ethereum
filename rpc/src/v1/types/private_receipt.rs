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

use v1::types::{H160, H256, TransactionRequest};
use ethcore_private_tx::{Receipt as EthPrivateReceipt};

/// Receipt
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateTransactionReceipt {
	/// Transaction Hash
	pub transaction_hash: H256,
	/// Private contract address
	pub contract_address: Option<H160>,
	/// Status code
	#[serde(rename = "status")]
	pub status_code: u8,
}

impl From<EthPrivateReceipt> for PrivateTransactionReceipt {
	fn from(r: EthPrivateReceipt) -> Self {
		PrivateTransactionReceipt {
			transaction_hash: r.hash.into(),
			contract_address: r.contract_address.map(Into::into),
			status_code: r.status_code.into(),
		}
	}
}

/// Receipt and Transaction
#[derive(Debug, Serialize)]
pub struct PrivateTransactionReceiptAndTransaction {
	/// Receipt
	pub receipt: PrivateTransactionReceipt,
	/// Transaction
	pub transaction: TransactionRequest,
}
