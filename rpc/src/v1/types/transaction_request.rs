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

use v1::types::{Bytes, H160, U256};
use v1::helpers;

use std::fmt;

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

impl fmt::Display for TransactionRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let eth = self.value.unwrap_or(U256::from(0));
		match self.to {
			Some(ref to) => write!(f, "{} Ether from {:?} to {:?}",
							   eth.format_ether(),
							   self.from,
							   to),
			None => write!(f, "{} Ether from {:?}",
							   eth.format_ether(),
							   self.from),
		}
	}
}

impl From<helpers::TransactionRequest> for TransactionRequest {
	fn from(r: helpers::TransactionRequest) -> Self {
		TransactionRequest {
			from: r.from.into(),
			to: r.to.map(Into::into),
			gas_price: r.gas_price.map(Into::into),
			gas: r.gas.map(Into::into),
			value: r.value.map(Into::into),
			data: r.data.map(Into::into),
			nonce: r.nonce.map(Into::into),
		}
	}
}

impl From<helpers::FilledTransactionRequest> for TransactionRequest {
	fn from(r: helpers::FilledTransactionRequest) -> Self {
		TransactionRequest {
			from: r.from.into(),
			to: r.to.map(Into::into),
			gas_price: Some(r.gas_price.into()),
			gas: Some(r.gas.into()),
			value: Some(r.value.into()),
			data: Some(r.data.into()),
			nonce: r.nonce.map(Into::into),
		}
	}
}

impl Into<helpers::TransactionRequest> for TransactionRequest {
	fn into(self) -> helpers::TransactionRequest {
		helpers::TransactionRequest {
			from: self.from.into(),
			to: self.to.map(Into::into),
			gas_price: self.gas_price.map(Into::into),
			gas: self.gas.map(Into::into),
			value: self.value.map(Into::into),
			data: self.data.map(Into::into),
			nonce: self.nonce.map(Into::into),
		}
	}
}


#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;
	use serde_json;
	use v1::types::{U256, H160};
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
			from: H160::from(1),
			to: Some(H160::from(2)),
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
			from: H160::from_str("b60e8dd61c5d32be8058bb8eb970870f07233155").unwrap(),
			to: Some(H160::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
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
			from: H160::from(1).into(),
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
			from: H160::from_str("b5f7502a2807cb23615c7456055e1d65b2508625").unwrap(),
			to: Some(H160::from_str("895d32f2db7d01ebb50053f9e48aacf26584fe40").unwrap()),
			gas_price: Some(U256::from_str("0ba43b7400").unwrap()),
			gas: Some(U256::from_str("2fd618").unwrap()),
			value: None,
			data: Some(vec![0x85, 0x95, 0xba, 0xb1].into()),
			nonce: None,
		});
	}

	#[test]
	fn transaction_request_deserialize_error() {
		let s = r#"{
			"from":"0xb5f7502a2807cb23615c7456055e1d65b2508625",
			"to":"",
			"data":"0x8595bab1",
			"gas":"0x2fd618",
			"gasPrice":"0x0ba43b7400"
		}"#;

		let deserialized = serde_json::from_str::<TransactionRequest>(s);

		assert!(deserialized.is_err(), "Should be error because to is empty");
	}
}
