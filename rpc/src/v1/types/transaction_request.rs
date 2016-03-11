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

use util::hash::Address;
use util::numbers::{Uint, U256};
use ethcore::transaction::{Action, Transaction};
use v1::types::Bytes;

#[derive(Debug, Default, PartialEq, Deserialize)]
pub struct TransactionRequest {
	pub from: Address,
	pub to: Option<Address>,
	#[serde(rename="gasPrice")]
	pub gas_price: Option<U256>,
	pub gas: Option<U256>,
	pub value: Option<U256>,
	pub data: Option<Bytes>,
	pub nonce: Option<U256>,
}

impl Into<Transaction> for TransactionRequest {
	fn into(self) -> Transaction {
		Transaction {
			nonce: self.nonce.unwrap_or_else(U256::zero),
			action: self.to.map_or(Action::Create, Action::Call),
			gas: self.gas.unwrap_or_else(U256::zero),
			gas_price: self.gas_price.unwrap_or_else(U256::zero),
			value: self.value.unwrap_or_else(U256::zero),
			data: self.data.map_or_else(Vec::new, |d| d.to_vec()),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use util::numbers::{Uint, U256};
	use util::hash::Address;
	use ethcore::transaction::{Transaction, Action};
	use v1::types::Bytes;
	use super::*;

	#[test]
	fn transaction_request_into_transaction() {
		let tr = TransactionRequest {
			from: Address::default(),
			to: Some(Address::from(10)),
			gas_price: Some(U256::from(20)),
			gas: Some(U256::from(10_000)),
			value: Some(U256::from(1)),
			data: Some(Bytes::new(vec![10, 20])),
			nonce: Some(U256::from(12)),
		};

		assert_eq!(Transaction {
			nonce: U256::from(12),
			action: Action::Call(Address::from(10)),
			gas: U256::from(10_000),
			gas_price: U256::from(20),
			value: U256::from(1),
			data: vec![10, 20],
		}, tr.into());
	}

	#[test]
	fn empty_transaction_request_into_transaction() {
		let tr = TransactionRequest {
			from: Address::default(),
			to: None,
			gas_price: None,
			gas: None,
			value: None,
			data: None,
			nonce: None,
		};

		assert_eq!(Transaction {
			nonce: U256::zero(),
			action: Action::Create,
			gas: U256::zero(),
			gas_price: U256::zero(),
			value: U256::zero(),
			data: vec![],
		}, tr.into());
	}

	#[test]
	fn transaction_request_deserialize() {
		let s = r#"{
			"from":"0x0000000000000000000000000000000000000001",
			"to":"0x0000000000000000000000000000000000000002",
			"gasPrice":"0x1",
			"gas":"0x2",
			"value":"0x3",
			"data":"0x123456",
			"nonce":"0x4"
		}"#;
		let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, TransactionRequest {
			from: Address::from(1),
			to: Some(Address::from(2)),
			gas_price: Some(U256::from(1)),
			gas: Some(U256::from(2)),
			value: Some(U256::from(3)),
			data: Some(Bytes::new(vec![0x12, 0x34, 0x56])),
			nonce: Some(U256::from(4)),
		});
	}

	#[test]
	fn transaction_request_deserialize_empty() {
		let s = r#"{"from":"0x0000000000000000000000000000000000000001"}"#;
		let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, TransactionRequest {
			from: Address::from(1),
			to: None,
			gas_price: None,
			gas: None,
			value: None,
			data: None,
			nonce: None,
		});
	}
}
