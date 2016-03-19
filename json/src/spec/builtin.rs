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

//! Spec builtin deserialization.

/// Linear pricing.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Linear {
	base: u64,
	word: u64,
}

/// Pricing variants.
#[derive(Debug, PartialEq, Deserialize)]
pub enum Pricing {
	/// Linear pricing.
	#[serde(rename="linear")]
	Linear(Linear),
}

/// Spec builtin.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Builtin {
	name: String,
	pricing: Pricing,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::builtin::Builtin;

	#[test]
	fn builtin_deserialization() {
		let s = r#"{
			"name": "ecrecover",
			"pricing": { "linear": { "base": 3000, "word": 0 } }
		}"#;
		let _deserialized: Builtin = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
