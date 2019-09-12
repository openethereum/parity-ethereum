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

/// Pricing for alt_bn128_pairing.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128Pairing {
	/// Base price.
	pub base: usize,
	/// Price per point pair.
	pub pair: usize,
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
}

/// Spec builtin.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Builtin {
	/// Builtin name
	pub name: String,
	/// One or several builtin prices
	pub price: BuiltinPrice,
}

/// Builtin price
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum BuiltinPrice {
	/// Single builtin
	Single(PricingWithActivation),
	/// Multiple builtins
	Multi(Vec<PricingWithActivation>),
}

/// Builtin price with which block to activate it on
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PricingWithActivation {
	/// Builtin pricing.
	pub pricing: Pricing,
	/// Activation block.
	pub activate_at: Option<u64>,
}

#[cfg(test)]
mod tests {
	use super::{Builtin, BuiltinPrice, Modexp, Linear, Pricing, PricingWithActivation};

	#[test]
	fn builtin_deserialization() {
		let s = r#"{
			"name": "ecrecover",
			"price": {
				"pricing": {
					"linear": {
						"base": 3000,
						"word": 0
					}
				}
			}
		}"#;
		let builtin: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(builtin.name, "ecrecover");
		assert_eq!(builtin.price, BuiltinPrice::Single(PricingWithActivation {
			pricing: Pricing::Linear(Linear {
				base: 3000,
				word: 0
			}),
			activate_at: None
		}));
	}

	#[test]
	fn deserialization_blake2_f_builtin() {
		let s = r#"{
			"name": "blake2_f",
			"price": {
				"pricing": {
					"blake2_f": {
						"gas_per_round": 123
					}
				},
				"activate_at": 16777215
			}
		}"#;
		let builtin: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(builtin.name, "blake2_f");
		assert_eq!(builtin.price, BuiltinPrice::Single(PricingWithActivation {
			pricing: Pricing::Blake2F {
				gas_per_round: 123,
			},
			activate_at: Some(0xffffff)
		}));
	}

	#[test]
	fn builtin_multi_deserialization() {
		let s = r#"{
			"name": "late_start",
			"price": [
				{
					"pricing": { "modexp": { "divisor": 5 } },
					"activate_at": 0
				},
				{
					"pricing": { "modexp": { "divisor": 5 } },
					"activate_at": 100000
				}
			]
		}"#;
		let builtin: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(builtin.name, "late_start");

		let expected = vec![
			PricingWithActivation {
				pricing: Pricing::Modexp(Modexp { divisor: 5 }),
				activate_at: Some(0)
			},
			PricingWithActivation {
				pricing: Pricing::Modexp(Modexp { divisor: 5 }),
				activate_at: Some(100_000)
			}
		];
		assert_eq!(builtin.price, BuiltinPrice::Multi(expected));
	}


	#[test]
	fn builtin_multi_deserialization_empty() {
		let s = r#"{
			"name": "builtin_with_price",
			"price": []
		}"#;
		let builtin: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(builtin.price, BuiltinPrice::Multi(Vec::new()));
	}
}
