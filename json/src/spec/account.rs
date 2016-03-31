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

//! Spec account deserialization.

use uint::Uint;
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
}

impl Account {
	/// Returns true if account does not have nonce and balance.
	pub fn is_empty(&self) -> bool {
		self.balance.is_none() && self.nonce.is_none()
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::account::Account;

	#[test]
	fn account_deserialization() {
		let s = r#"{
			"balance": "1",
			"builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } }
		}"#;
		let _deserialized: Account = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
