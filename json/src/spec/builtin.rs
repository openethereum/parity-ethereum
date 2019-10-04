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

/// Pricing for constant alt_bn128 operations
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128ConstOperations {
	/// price
	pub price: usize,
	/// EIP 1108 transition price
	// for backward compability
	pub eip1108_transition_price: Option<usize>,
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
	// for backward compability
	pub eip1108_transition_base: Option<usize>,
	/// EIP 1108 transition price per point pair
	// for backward compability
	pub eip1108_transition_pair: Option<usize>,
}

/// Pricing variants.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum InnerPricing {
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
	// for backward comptabiltity
	pub eip1108_transition: Option<Uint>,
}

/// Builtin price
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Pricing {
	/// Single builtin
	Single(InnerPricing),
	/// Multiple builtins
	Multi(Vec<PricingWithActivation>),
}

/// Builtin price with which block to activate it on
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PricingWithActivation {
	/// Description of the activation
	pub info: Option<String>,
	/// Builtin pricing.
	pub price: InnerPricing,
	/// Activation block.
	pub activate_at: Uint,
}

#[cfg(test)]
mod tests {
	use super::{Builtin, Pricing, InnerPricing, PricingWithActivation, Uint, Linear, Modexp, AltBn128ConstOperations};

	#[test]
	fn builtin_deserialization() {
		let s = r#"{
			"name": "ecrecover",
			"pricing": { "linear": { "base": 3000, "word": 0 } }
		}"#;
		let deserialized: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.name, "ecrecover");
		assert_eq!(deserialized.pricing, Pricing::Single(InnerPricing::Linear(Linear { base: 3000, word: 0 })));
		assert!(deserialized.activate_at.is_none());
	}

	#[test]
	fn builtin_v2_deserialization() {
		let s = r#"{
			"name": "ecrecover",
			"pricing": [
				{
					"activate_at": 0,
					"price": {"linear": { "base": 3000, "word": 0 }}
				},
				{
					"info": "enable fake EIP at block 500",
					"activate_at": 500,
					"price": {"linear": { "base": 10, "word": 0 }}
				}
			]
		}"#;
		let deserialized: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.name, "ecrecover");
		assert_eq!(deserialized.pricing, Pricing::Multi(vec![
			PricingWithActivation {
				info: None,
				activate_at: Uint(0.into()),
				price: InnerPricing::Linear(Linear { base: 3000, word: 0 })
			},
			PricingWithActivation {
				info: Some(String::from("enable fake EIP at block 500")),
				activate_at: Uint(500.into()),
				price: InnerPricing::Linear(Linear { base: 10, word: 0 })
			}
		]));
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
		assert_eq!(deserialized.pricing, Pricing::Single(InnerPricing::Blake2F { gas_per_round: 123 }));
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
		assert_eq!(deserialized.pricing, Pricing::Single(InnerPricing::Modexp(Modexp { divisor: 5 })));
		assert_eq!(deserialized.activate_at, Some(Uint(100000.into())));
	}

	#[test]
	fn optional_eip1108_fields() {
		let s = r#"{
			"name": "alt_bn128_add",
			"activate_at": "0x00",
			"eip1108_transition": "0x17d433",
			"pricing": {
				"alt_bn128_const_operations": {
					"price": 500,
					"eip1108_transition_price": 150
				}
			}
		}"#;
		let deserialized: Builtin = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.name, "alt_bn128_add");
		assert_eq!(
			deserialized.pricing,
			Pricing::Single(InnerPricing::AltBn128ConstOperations(AltBn128ConstOperations {
				price: 500,
				eip1108_transition_price: Some(150),
			}))
		);
		assert_eq!(deserialized.activate_at, Some(Uint(0.into())));
		assert_eq!(deserialized.eip1108_transition, Some(Uint(0x17d433.into())));
	}
}
