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

//! `TransactionRequest` type

use util::U256;
use v1::types::{Bytes, H160};
use v1::helpers::{TransactionRequest as Request, TransactionConfirmation as Confirmation};

/// Transaction request coming from RPC
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TransactionRequest {
	/// Sender
	pub from: H160,
	/// Recipient
	pub to: Option<H160>,
	/// Gas Price
	#[serde(rename="gasPrice")]
	pub gas_price: Option<U256>,
	/// Gas
	pub gas: Option<U256>,
	/// Value of transaction in wei
	pub value: Option<U256>,
	/// Additional data sent with transaction
	pub data: Option<Bytes>,
	/// Transaction's nonce
	pub nonce: Option<U256>,
}

impl From<Request> for TransactionRequest {
	fn from(r: Request) -> Self {
		TransactionRequest {
			from: r.from.into(),
			to: r.to.map(Into::into),
			gas_price: r.gas_price,
			gas: r.gas,
			value: r.value,
			data: r.data.map(Into::into),
			nonce: r.nonce
		}
	}
}

impl Into<Request> for TransactionRequest {
	fn into(self) -> Request {
		Request {
			from: self.from.into(),
			to: self.to.map(Into::into),
			gas_price: self.gas_price,
			gas: self.gas,
			value: self.value,
			data: self.data.map(Into::into),
			nonce: self.nonce
		}
	}
}

/// Transaction confirmation waiting in a queue
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize)]
pub struct TransactionConfirmation {
	/// Id of this confirmation
	pub id: U256,
	/// TransactionRequest
	pub transaction: TransactionRequest,
}

impl From<Confirmation> for TransactionConfirmation {
	fn from(c: Confirmation) -> Self {
		TransactionConfirmation {
			id: c.id,
			transaction: c.transaction.into(),
		}
	}
}

/// Possible modifications to the confirmed transaction sent by `SignerUI`
#[derive(Debug, PartialEq, Deserialize)]
pub struct TransactionModification {
	/// Modified gas price
	#[serde(rename="gasPrice")]
	pub gas_price: Option<U256>,
}


#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;
	use serde_json;
	use util::{U256, Address};
	use super::*;

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
			from: Address::from(1).into(),
			to: Some(Address::from(2).into()),
			gas_price: Some(U256::from(1)),
			gas: Some(U256::from(2)),
			value: Some(U256::from(3)),
			data: Some(vec![0x12, 0x34, 0x56].into()),
			nonce: Some(U256::from(4)),
		});
	}

	#[test]
	fn transaction_request_deserialize2() {
		let s = r#"{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		}"#;
		let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, TransactionRequest {
			from: Address::from_str("b60e8dd61c5d32be8058bb8eb970870f07233155").unwrap().into(),
			to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap().into()),
			gas_price: Some(U256::from_str("9184e72a000").unwrap()),
			gas: Some(U256::from_str("76c0").unwrap()),
			value: Some(U256::from_str("9184e72a").unwrap()),
			data: Some("d46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675".from_hex().unwrap().into()),
			nonce: None
		});
	}

	#[test]
	fn transaction_request_deserialize_empty() {
		let s = r#"{"from":"0x0000000000000000000000000000000000000001"}"#;
		let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, TransactionRequest {
			from: Address::from(1).into(),
			to: None,
			gas_price: None,
			gas: None,
			value: None,
			data: None,
			nonce: None,
		});
	}

	#[test]
	fn transaction_request_deserialize_test() {
		let s = r#"{
			"from":"0xb5f7502a2807cb23615c7456055e1d65b2508625",
			"to":"0x895d32f2db7d01ebb50053f9e48aacf26584fe40",
			"data":"0x8595bab1",
			"gas":"0x2fd618",
			"gasPrice":"0x0ba43b7400"
		}"#;

		let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, TransactionRequest {
			from: Address::from_str("b5f7502a2807cb23615c7456055e1d65b2508625").unwrap().into(),
			to: Some(Address::from_str("895d32f2db7d01ebb50053f9e48aacf26584fe40").unwrap().into()),
			gas_price: Some(U256::from_str("0ba43b7400").unwrap()),
			gas: Some(U256::from_str("2fd618").unwrap()),
			value: None,
			data: Some(vec![0x85, 0x95, 0xba, 0xb1].into()),
			nonce: None,
		});
	}

	#[test]
	fn should_deserialize_modification() {
		// given
		let s1 = r#"{
			"gasPrice":"0x0ba43b7400"
		}"#;
		let s2 = r#"{}"#;

		// when
		let res1: TransactionModification = serde_json::from_str(s1).unwrap();
		let res2: TransactionModification = serde_json::from_str(s2).unwrap();

		// then
		assert_eq!(res1, TransactionModification {
			gas_price: Some(U256::from_str("0ba43b7400").unwrap()),
		});
		assert_eq!(res2, TransactionModification {
			gas_price: None,
		});
	}
}

