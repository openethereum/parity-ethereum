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
use util::numbers::U256;
use v1::types::Bytes;

#[derive(Debug, Default, PartialEq, Deserialize)]
pub struct CallRequest {
	pub from: Option<Address>,
	pub to: Option<Address>,
	#[serde(rename="gasPrice")]
	pub gas_price: Option<U256>,
	pub gas: Option<U256>,
	pub value: Option<U256>,
	pub data: Option<Bytes>,
	pub nonce: Option<U256>,
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;
	use serde_json;
	use util::numbers::{U256};
	use util::hash::Address;
	use v1::types::Bytes;
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
		let deserialized: CallRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, CallRequest {
			from: Some(Address::from(1)),
			to: Some(Address::from(2)),
			gas_price: Some(U256::from(1)),
			gas: Some(U256::from(2)),
			value: Some(U256::from(3)),
			data: Some(Bytes::new(vec![0x12, 0x34, 0x56])),
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
		let deserialized: CallRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, CallRequest {
			from: Some(Address::from_str("b60e8dd61c5d32be8058bb8eb970870f07233155").unwrap()),
			to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
			gas_price: Some(U256::from_str("9184e72a000").unwrap()),
			gas: Some(U256::from_str("76c0").unwrap()),
			value: Some(U256::from_str("9184e72a").unwrap()),
			data: Some(Bytes::new("d46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675".from_hex().unwrap())),
			nonce: None
		});
	}

	#[test]
	fn transaction_request_deserialize_empty() {
		let s = r#"{"from":"0x0000000000000000000000000000000000000001"}"#;
		let deserialized: CallRequest = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, CallRequest {
			from: Some(Address::from(1)),
			to: None,
			gas_price: None,
			gas: None,
			value: None,
			data: None,
			nonce: None,
		});
	}
}
