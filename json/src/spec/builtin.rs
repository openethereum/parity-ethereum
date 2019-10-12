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

use std::collections::BTreeMap;

use crate::uint::Uint;
use serde::Deserialize;

/// Linear pricing.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Linear {
	/// Base price.
	pub base: u64,
	/// Price for word.
	pub word: u64,
}

/// Pricing for modular exponentiation.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Modexp {
	/// Price divisor.
	pub divisor: u64,
}

/// Pricing for constant alt_bn128 operations (ECADD and ECMUL)
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128ConstOperations {
	/// price
	pub price: u64,
	/// EIP 1108 transition price
	// for backward compatibility
	pub eip1108_transition_price: Option<u64>,
}

impl AltBn128ConstOperations {
	fn clear_eip1108(&mut self) {
		self.eip1108_transition_price = None;
	}
}

/// Pricing for alt_bn128_pairing.
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128Pairing {
	/// Base price.
	pub base: u64,
	/// Price per point pair.
	pub pair: u64,
	/// EIP 1108 transition base price
	// for backward compatibility
	pub eip1108_transition_base: Option<u64>,
	/// EIP 1108 transition price per point pair
	// for backward compatibility
	pub eip1108_transition_pair: Option<u64>,
}

impl AltBn128Pairing {
	fn clear_eip1108(&mut self) {
		self.eip1108_transition_base = None;
		self.eip1108_transition_pair = None;
	}

	fn eip1108_fields(&self) -> Option<(u64, u64)> {
		if let (Some(b), Some(p)) = (self.eip1108_transition_base, self.eip1108_transition_pair) {
			Some((b, p))
		} else {
			None
		}
	}
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

/// Builtin compability layer
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BuiltinCompat {
	/// Builtin name.
	name: String,
	/// Builtin pricing.
	pricing: PricingCompat,
	/// Activation block.
	activate_at: Option<Uint>,
	/// EIP 1108
	// for backward compatibility
	eip1108_transition: Option<Uint>,
}

/// Spec builtin.
#[derive(Debug, PartialEq, Clone)]
pub struct Builtin {
	/// Builtin name.
	pub name: String,
	/// Builtin pricing.
	pub pricing: BTreeMap<u64, PricingAt>,
}

impl From<BuiltinCompat> for Builtin {
	fn from(legacy: BuiltinCompat) -> Self {
		let pricing: BTreeMap<u64, PricingAt> = match legacy.pricing {
			PricingCompat::Single(pricing) => {
				let mut map: BTreeMap<u64, PricingAt> = BTreeMap::new();
				let activate_at: u64 = legacy.activate_at.map_or(0, Into::into);

				match pricing {
					Pricing::AltBn128Pairing(mut p) => {
						let mut pricing = p.clone();

						pricing.clear_eip1108();
						map.insert(activate_at, PricingAt {
							info: None,
							price: Pricing::AltBn128Pairing(pricing),
						});

						if let (Some(a), Some(pairing)) = (legacy.eip1108_transition, p.eip1108_fields()) {
							p.clear_eip1108();
							p.base = pairing.0;
							p.pair = pairing.1;
							map.insert(a.into(), PricingAt {
								info: Some("EIP1108 transition".to_string()),
								price: Pricing::AltBn128Pairing(p),
							});
						}
					}
					Pricing::AltBn128ConstOperations(mut p) => {
						let mut pricing = p.clone();

						pricing.clear_eip1108();
						map.insert(activate_at, PricingAt {
							info: None,
							price: Pricing::AltBn128ConstOperations(pricing),
						});

						if let (Some(a), Some(price)) = (legacy.eip1108_transition, p.eip1108_transition_price) {
							p.clear_eip1108();
							p.price = price;
							map.insert(a.into(), PricingAt {
								info: Some("EIP1108 transition".to_string()),
								price: Pricing::AltBn128ConstOperations(p),
							});
						}
					}
					price => {
						let activate_at: u64 = legacy.activate_at.map_or(0, Into::into);
						map.insert(activate_at, PricingAt { info: None, price });
					}
				};
				map
			}
			PricingCompat::Multi(pricings) => {
				pricings.into_iter().map(|(a, p)| (a.into(), p)).collect()
			}
		};
		Self { name: legacy.name, pricing }
	}
}

/// Compability layer for different pricings
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
enum PricingCompat {
	/// Single builtin
	Single(Pricing),
	/// Multiple builtins
	Multi(BTreeMap<Uint, PricingAt>),
}

/// Builtin price with which block to activate it on
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PricingAt {
	/// Description of the activation
	pub info: Option<String>,
	/// Builtin pricing.
	pub price: Pricing,
}

#[cfg(test)]
mod tests {
	use super::{Builtin, BuiltinCompat, BTreeMap, Pricing, PricingAt, Linear, Modexp, AltBn128ConstOperations};
	use macros::map;

	#[test]
	fn builtin_deserialization() {
		let s = r#"{
			"name": "ecrecover",
			"pricing": { "linear": { "base": 3000, "word": 0 } }
		}"#;
		let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
		assert_eq!(builtin.name, "ecrecover");
		assert_eq!(builtin.pricing, map![
			0 => PricingAt {
				info: None,
				price: Pricing::Linear(Linear { base: 3000, word: 0 })
			}
		]);
	}

	#[test]
	fn deserialize_multiple_pricings() {
		let s = r#"{
			"name": "ecrecover",
			"pricing": {
				"0": {
					"price": {"linear": { "base": 3000, "word": 0 }}
				},
				"500": {
					"info": "enable fake EIP at block 500",
					"price": {"linear": { "base": 10, "word": 0 }}
				}
			}
		}"#;
		let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
		assert_eq!(builtin.name, "ecrecover");
		assert_eq!(builtin.pricing, map![
			0 => PricingAt {
				info: None,
				price: Pricing::Linear(Linear { base: 3000, word: 0 })
			},
			500 => PricingAt {
				info: Some(String::from("enable fake EIP at block 500")),
				price: Pricing::Linear(Linear { base: 10, word: 0 })
			}
		]);
	}

	#[test]
	fn deserialization_blake2_f_builtin() {
		let s = r#"{
			"name": "blake2_f",
			"activate_at": "0xffffff",
			"pricing": { "blake2_f": { "gas_per_round": 123 } }
		}"#;
		let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
		assert_eq!(builtin.name, "blake2_f");
		assert_eq!(builtin.pricing, map![
			0xffffff => PricingAt {
				info: None,
				price: Pricing::Blake2F { gas_per_round: 123 }
			}
		]);
	}

	#[test]
	fn activate_at() {
		let s = r#"{
			"name": "late_start",
			"activate_at": 100000,
			"pricing": { "modexp": { "divisor": 5 } }
		}"#;

		let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
		assert_eq!(builtin.name, "late_start");
		assert_eq!(builtin.pricing, map![
			100_000 => PricingAt {
				info: None,
				price: Pricing::Modexp(Modexp { divisor: 5 })
			}
		]);
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
		let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
		assert_eq!(builtin.name, "alt_bn128_add");
		assert_eq!(builtin.pricing, map![
			0 => PricingAt {
				info: None,
				price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
					price: 500,
					eip1108_transition_price: None
				})
			},
			0x17d433 => PricingAt {
				info: Some("EIP1108 transition".to_string()),
				price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
					price: 150,
					eip1108_transition_price: None
				})
			}
		]);
	}
}
