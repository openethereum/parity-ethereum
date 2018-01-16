// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Spec account deserialization.

use std::collections::BTreeMap;
use uint::Uint;
use bytes::Bytes;
use spec::builtin::Builtin;

/// Spec account.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Account {
	/// Builtin contract.
	pub builtin: Option<Builtin>,
	/// Balance.
	pub balance: Option<Uint>,
	/// Nonce.
	pub nonce: Option<Uint>,
	/// Code.
	pub code: Option<Bytes>,
	/// Storage.
	pub storage: Option<BTreeMap<Uint, Uint>>,
	/// Constructor.
	pub constructor: Option<Bytes>,
}

impl Account {
	/// Returns true if account does not have nonce, balance, code and storage.
	pub fn is_empty(&self) -> bool {
		self.balance.is_none() && self.nonce.is_none() && self.code.is_none() && self.storage.is_none()
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use serde_json;
	use spec::account::Account;
	use ethereum_types::U256;
	use uint::Uint;
	use bytes::Bytes;

	#[test]
	fn account_balance_missing_not_empty() {
		let s = r#"{
			"nonce": "0",
			"code": "1234",
			"storage": { "0x7fffffffffffffff7fffffffffffffff": "0x1" }
		}"#;
		let deserialized: Account = serde_json::from_str(s).unwrap();
		assert!(!deserialized.is_empty());
	}

	#[test]
	fn account_nonce_missing_not_empty() {
		let s = r#"{
			"balance": "1",
			"code": "1234",
			"storage": { "0x7fffffffffffffff7fffffffffffffff": "0x1" }
		}"#;
		let deserialized: Account = serde_json::from_str(s).unwrap();
		assert!(!deserialized.is_empty());
	}

	#[test]
	fn account_code_missing_not_empty() {
		let s = r#"{
			"balance": "1",
			"nonce": "0",
			"storage": { "0x7fffffffffffffff7fffffffffffffff": "0x1" }
		}"#;
		let deserialized: Account = serde_json::from_str(s).unwrap();
		assert!(!deserialized.is_empty());
	}

	#[test]
	fn account_storage_missing_not_empty() {
		let s = r#"{
			"balance": "1",
			"nonce": "0",
			"code": "1234"
		}"#;
		let deserialized: Account = serde_json::from_str(s).unwrap();
		assert!(!deserialized.is_empty());
	}

	#[test]
	fn account_empty() {
		let s = r#"{
			"builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } }
		}"#;
		let deserialized: Account = serde_json::from_str(s).unwrap();
		assert!(deserialized.is_empty());
	}

	#[test]
	fn account_deserialization() {
		let s = r#"{
			"balance": "1",
			"nonce": "0",
			"builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } },
			"code": "1234"
		}"#;
		let deserialized: Account = serde_json::from_str(s).unwrap();
		assert!(!deserialized.is_empty());
		assert_eq!(deserialized.balance.unwrap(), Uint(U256::from(1)));
		assert_eq!(deserialized.nonce.unwrap(), Uint(U256::from(0)));
		assert_eq!(deserialized.code.unwrap(), Bytes::new(vec![0x12, 0x34]));
		assert!(deserialized.builtin.is_some()); // Further tested in builtin.rs
	}

	#[test]
	fn account_storage_deserialization() {
		let s = r#"{
			"balance": "1",
			"nonce": "0",
			"code": "1234",
			"storage": { "0x7fffffffffffffff7fffffffffffffff": "0x1" }
		}"#;
		let deserialized: Account = serde_json::from_str(s).unwrap();
		assert!(!deserialized.is_empty());
		assert_eq!(deserialized.balance.unwrap(), Uint(U256::from(1)));
		assert_eq!(deserialized.nonce.unwrap(), Uint(U256::from(0)));
		assert_eq!(deserialized.code.unwrap(), Bytes::new(vec![0x12, 0x34]));
		let mut storage = BTreeMap::new();
		storage.insert(Uint(U256::from("7fffffffffffffff7fffffffffffffff")), Uint(U256::from(1)));
		assert_eq!(deserialized.storage.unwrap(), storage);
	}
}
