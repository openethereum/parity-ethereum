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

use util::numbers::*;
use ethcore::contract_address;
use ethcore::transaction::{LocalizedTransaction, Action, SignedTransaction};
use v1::types::{Bytes, OptionalValue};

/// Transaction
#[derive(Debug, Default, Serialize)]
pub struct Transaction {
	/// Hash
	pub hash: H256,
	/// Nonce
	pub nonce: U256,
	/// Block hash
	#[serde(rename="blockHash")]
	pub block_hash: OptionalValue<H256>,
	/// Block number
	#[serde(rename="blockNumber")]
	pub block_number: OptionalValue<U256>,
	/// Transaction Index
	#[serde(rename="transactionIndex")]
	pub transaction_index: OptionalValue<U256>,
	/// Sender
	pub from: Address,
	/// Recipient
	pub to: OptionalValue<Address>,
	/// Transfered value
	pub value: U256,
	/// Gas Price
	#[serde(rename="gasPrice")]
	pub gas_price: U256,
	/// Gas
	pub gas: U256,
	/// Data
	pub input: Bytes,
	/// Creates contract
	pub creates: OptionalValue<Address>,
}

impl From<LocalizedTransaction> for Transaction {
	fn from(t: LocalizedTransaction) -> Transaction {
		Transaction {
			hash: t.hash(),
			nonce: t.nonce,
			block_hash: OptionalValue::Value(t.block_hash.clone()),
			block_number: OptionalValue::Value(U256::from(t.block_number)),
			transaction_index: OptionalValue::Value(U256::from(t.transaction_index)),
			from: t.sender().unwrap(),
			to: match t.action {
				Action::Create => OptionalValue::Null,
				Action::Call(ref address) => OptionalValue::Value(address.clone())
			},
			value: t.value,
			gas_price: t.gas_price,
			gas: t.gas,
			input: Bytes::new(t.data.clone()),
			creates: match t.action {
				Action::Create => OptionalValue::Value(contract_address(&t.sender().unwrap(), &t.nonce)),
				Action::Call(_) => OptionalValue::Null,
			},
		}
	}
}

impl From<SignedTransaction> for Transaction {
	fn from(t: SignedTransaction) -> Transaction {
		Transaction {
			hash: t.hash(),
			nonce: t.nonce,
			block_hash: OptionalValue::Null,
			block_number: OptionalValue::Null,
			transaction_index: OptionalValue::Null,
			from: t.sender().unwrap(),
			to: match t.action {
				Action::Create => OptionalValue::Null,
				Action::Call(ref address) => OptionalValue::Value(address.clone())
			},
			value: t.value,
			gas_price: t.gas_price,
			gas: t.gas,
			input: Bytes::new(t.data.clone()),
			creates: match t.action {
				Action::Create => OptionalValue::Value(contract_address(&t.sender().unwrap(), &t.nonce)),
				Action::Call(_) => OptionalValue::Null,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn test_transaction_serialize() {
		let t = Transaction::default();
		let serialized = serde_json::to_string(&t).unwrap();
		// TODO: fix create.
		assert_eq!(serialized, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x00","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x0000000000000000000000000000000000000000","to":null,"value":"0x00","gasPrice":"0x00","gas":"0x00","input":"0x","creates":null}"#);
	}
}

