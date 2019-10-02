// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Spec builtin deserialization.

use crate::uint::Uint;
use serde::Deserialize;


/// Linear pricing.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Linear {
	/// Base price.
	pub base: usize,
	/// Price for word.
	pub word: usize,
}

/// Pricing for modular exponentiation.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Modexp {
	/// Price divisor.
	pub divisor: usize,
}

/// Pricing for constant alt_bn128 operations (ECADD and ECMUL)
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128ConstOperations {
	/// price
	pub price: usize,
	/// EIP 1108 transition price
	pub eip1108_transition_price: usize,
}

/// Pricing for alt_bn128_pairing.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128Pairing {
	/// Base price.
	pub base: usize,
	/// Price per point pair.
	pub pair: usize,
	/// EIP 1108 transition base price
	pub eip1108_transition_base: usize,
	/// EIP 1108 transition price per point pair
	pub eip1108_transition_pair: usize,
}

/// Pricing variants.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum Pricing {
	/// Pricing for Blake2 compression function: each call costs the same amount per round.
	Blake2F {
		/// Price per round of Blake2 compression function.
		gas_per_round: u64,
	},
	/// Linear pricing.
	Linear(Linear),
	/// Pricing for modular exponentiation.
	Modexp(Modexp),
	/// Pricing for alt_bn128_pairing exponentiation.
	AltBn128Pairing(AltBn128Pairing),
	/// Pricing for constant alt_bn128 operations
	AltBn128ConstOperations(AltBn128ConstOperations),
}

/// Spec builtin.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Builtin {
	/// Builtin name.
	pub name: String,
	/// Builtin pricing.
	pub pricing: Pricing,
	/// Activation block.
	pub activate_at: Option<Uint>,
	/// EIP 1108
	pub eip1108_transition: Option<Uint>,
}

#[cfg(test)]
mod tests {
	use super::{Builtin, Modexp, Linear, Pricing, Uint};

	#[test]
	fn builtin_deserialization() {
		let s = r#"{
			"name": "ecrecover",
			"pricing": { "linear": { "base": 3000, "word": 0 } }
		}"#;
		let deserialized: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.name, "ecrecover");
		assert_eq!(deserialized.pricing, Pricing::Linear(Linear { base: 3000, word: 0 }));
		assert!(deserialized.activate_at.is_none());
	}

	#[test]
	fn deserialization_blake2_f_builtin() {
		let s = r#"{
			"name": "blake2_f",
			"activate_at": "0xffffff",
			"pricing": { "blake2_f": { "gas_per_round": 123 } }
		}"#;
		let deserialized: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.name, "blake2_f");
		assert_eq!(deserialized.pricing, Pricing::Blake2F { gas_per_round: 123 });
		assert!(deserialized.activate_at.is_some());
	}

	#[test]
	fn activate_at() {
		let s = r#"{
			"name": "late_start",
			"activate_at": 100000,
			"pricing": { "modexp": { "divisor": 5 } }
		}"#;

		let deserialized: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.name, "late_start");
		assert_eq!(deserialized.pricing, Pricing::Modexp(Modexp { divisor: 5 }));
		assert_eq!(deserialized.activate_at, Some(Uint(100000.into())));
	}
}
