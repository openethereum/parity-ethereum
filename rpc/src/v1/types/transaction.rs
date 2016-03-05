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
use ethcore::transaction::{LocalizedTransaction, Action};
use v1::types::{Bytes, OptionalValue};
use serde::{Deserializer, Error};
use ethcore;
use util;

#[derive(Debug, Default, Serialize)]
pub struct Transaction {
	pub hash: H256,
	pub nonce: U256,
	#[serde(rename="blockHash")]
	pub block_hash: OptionalValue<H256>,
	#[serde(rename="blockNumber")]
	pub block_number: OptionalValue<U256>,
	#[serde(rename="transactionIndex")]
	pub transaction_index: OptionalValue<U256>,
	pub from: Address,
	pub to: OptionalValue<Address>,
	pub value: U256,
	#[serde(rename="gasPrice")]
	pub gas_price: U256,
	pub gas: U256,
	pub input: Bytes
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransactionRequest {
	pub from: Address,
	pub to: Option<Address>,
	#[serde(rename="gasPrice")]
	pub gas_price: Option<U256>,
	pub gas: Option<U256>,
	pub value: Option<U256>,
	pub data: Bytes,
	pub nonce: Option<U256>,
}

impl TransactionRequest {
	fn to_eth(self) -> (ethcore::transaction::Transaction, Address) {
		(ethcore::transaction::Transaction {
			nonce: self.nonce.unwrap_or(U256::zero()),
			action: match self.to {
				None => ethcore::transaction::Action::Create,
				Some(addr) => ethcore::transaction::Action::Call(addr)
			},
			gas: self.gas.unwrap_or(U256::zero()),
			gas_price: self.gas_price.unwrap_or(U256::zero()),
			value: self.value.unwrap_or(U256::zero()),
			data: { let (ref x) = self.data; x }
		}, self.from)
	}
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
			input: Bytes::new(t.data.clone())
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
		assert_eq!(serialized, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x00","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x0000000000000000000000000000000000000000","to":null,"value":"0x00","gasPrice":"0x00","gas":"0x00","input":"0x00"}"#);
	}
}

