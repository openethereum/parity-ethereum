// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use ethereum_types::{H256, U256};
use serde_json;

type Res = Result<U256, serde_json::Error>;

#[test]
fn should_serialize_u256() {
	let serialized1 = serde_json::to_string(&U256::from(0)).unwrap();
	let serialized2 = serde_json::to_string(&U256::from(1)).unwrap();
	let serialized3 = serde_json::to_string(&U256::from(16)).unwrap();
	let serialized4 = serde_json::to_string(&U256::from(256)).unwrap();

	assert_eq!(serialized1, r#""0x0""#);
	assert_eq!(serialized2, r#""0x1""#);
	assert_eq!(serialized3, r#""0x10""#);
	assert_eq!(serialized4, r#""0x100""#);
}

#[test]
fn should_serialize_h256() {
	let serialized1 = serde_json::to_string(&H256::from_low_u64_be(0)).unwrap();
	let serialized2 = serde_json::to_string(&H256::from_low_u64_be(1)).unwrap();
	let serialized3 = serde_json::to_string(&H256::from_low_u64_be(16)).unwrap();
	let serialized4 = serde_json::to_string(&H256::from_low_u64_be(256)).unwrap();

	assert_eq!(serialized1, r#""0x0000000000000000000000000000000000000000000000000000000000000000""#);
	assert_eq!(serialized2, r#""0x0000000000000000000000000000000000000000000000000000000000000001""#);
	assert_eq!(serialized3, r#""0x0000000000000000000000000000000000000000000000000000000000000010""#);
	assert_eq!(serialized4, r#""0x0000000000000000000000000000000000000000000000000000000000000100""#);
}

#[test]
fn should_fail_to_deserialize_decimals() {
	let deserialized0: Res = serde_json::from_str(r#""∀∂""#);
	let deserialized1: Res = serde_json::from_str(r#""""#);
	let deserialized2: Res = serde_json::from_str(r#""0""#);
	let deserialized3: Res = serde_json::from_str(r#""10""#);
	let deserialized4: Res = serde_json::from_str(r#""1000000""#);
	let deserialized5: Res = serde_json::from_str(r#""1000000000000000000""#);
	let deserialized6: Res = serde_json::from_str(r#""0x""#);

	assert!(deserialized0.is_err());
	assert!(deserialized1.is_err());
	assert!(deserialized2.is_err());
	assert!(deserialized3.is_err());
	assert!(deserialized4.is_err());
	assert!(deserialized5.is_err());
	assert!(deserialized6.is_err(), "Quantities should represent zero as 0x0");
}

#[test]
fn should_fail_to_deserialize_bad_hex_strings() {
	let deserialized1: Result<H256, serde_json::Error> = serde_json::from_str(r#""0""#);
	let deserialized2: Result<H256, serde_json::Error> = serde_json::from_str(r#""0x""#);
	let deserialized3: Result<H256, serde_json::Error> = serde_json::from_str(r#""0x∀∂0000000000000000000000000000000000000000000000000000000000""#);

	assert!(deserialized1.is_err(), "hex string should start with 0x");
	assert!(deserialized2.is_err(), "0x-prefixed hex string of length 64");
	assert!(deserialized3.is_err(), "hex string should only contain hex chars");
}

#[test]
fn should_deserialize_u256() {
	let deserialized1: U256 = serde_json::from_str(r#""0x0""#).unwrap();
	let deserialized2: U256 = serde_json::from_str(r#""0x1""#).unwrap();
	let deserialized3: U256 = serde_json::from_str(r#""0x01""#).unwrap();
	let deserialized4: U256 = serde_json::from_str(r#""0x100""#).unwrap();

	assert_eq!(deserialized1, 0.into());
	assert_eq!(deserialized2, 1.into());
	assert_eq!(deserialized3, 1.into());
	assert_eq!(deserialized4, 256.into());
}

#[test]
fn should_deserialize_h256() {
	let deserialized1: H256 = serde_json::from_str(r#""0x0000000000000000000000000000000000000000000000000000000000000000""#).unwrap();
	let deserialized2: H256 = serde_json::from_str(r#""0x0000000000000000000000000000000000000000000000000000000000000001""#).unwrap();
	let deserialized3: H256 = serde_json::from_str(r#""0x0000000000000000000000000000000000000000000000000000000000000100""#).unwrap();

	assert_eq!(deserialized1, H256::from_low_u64_be(0));
	assert_eq!(deserialized2, H256::from_low_u64_be(1));
	assert_eq!(deserialized3, H256::from_low_u64_be(256));
}
