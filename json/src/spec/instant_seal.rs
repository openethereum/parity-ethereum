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

//! Instant params deserialization.

use hash::Address;

/// Instant params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct InstantSealParams {
	/// Address of the registrar contract.
	pub registrar: Option<Address>,
}

/// Instant engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct InstantSeal {
	/// Instant Seal params.
	pub params: InstantSealParams,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::instant_seal::InstantSeal;

	#[test]
	fn instant_seal_deserialization() {
		let s = r#"{
			"params": {
				"registrar": "0xc6d9d2cd449a754c494264e1809c50e34d64562b"
			}
		}"#;

		let _deserialized: InstantSeal = serde_json::from_str(s).unwrap();
	}
}
