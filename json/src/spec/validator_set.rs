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

//! Validator set deserialization.

use std::collections::BTreeMap;
use uint::Uint;
use hash::Address;

/// Different ways of specifying validators.
#[derive(Debug, PartialEq, Deserialize)]
pub enum ValidatorSet {
	/// A simple list of authorities.
	#[serde(rename="list")]
	List(Vec<Address>),
	/// Address of a contract that indicates the list of authorities.
	#[serde(rename="safeContract")]
	SafeContract(Address),
	/// Address of a contract that indicates the list of authorities and enables reporting of theor misbehaviour using transactions.
	#[serde(rename="contract")]
	Contract(Address),
	/// A map of starting blocks for each validator set.
	#[serde(rename="multi")]
	Multi(BTreeMap<Uint, ValidatorSet>),
}

#[cfg(test)]
mod tests {
	use serde_json;
	use uint::Uint;
	use ethereum_types::{H160, U256};
	use hash::Address;
	use spec::validator_set::ValidatorSet;

	#[test]
	fn validator_set_deserialization() {
		let s = r#"[{
			"list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
		}, {
			"safeContract": "0xc6d9d2cd449a754c494264e1809c50e34d64562b"
		}, {
			"contract": "0xc6d9d2cd449a754c494264e1809c50e34d64562b"
		}, {
			"multi": {
				"0": { "list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"] },
				"10": { "list": ["0xd6d9d2cd449a754c494264e1809c50e34d64562b"] },
				"20": { "contract": "0xc6d9d2cd449a754c494264e1809c50e34d64562b" }
			}
		}]"#;

		let deserialized: Vec<ValidatorSet> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.len(), 4);

		assert_eq!(deserialized[0], ValidatorSet::List(vec![Address(H160::from("0xc6d9d2cd449a754c494264e1809c50e34d64562b"))]));
		assert_eq!(deserialized[1], ValidatorSet::SafeContract(Address(H160::from("0xc6d9d2cd449a754c494264e1809c50e34d64562b"))));
		assert_eq!(deserialized[2], ValidatorSet::Contract(Address(H160::from("0xc6d9d2cd449a754c494264e1809c50e34d64562b"))));
		match deserialized[3] {
			ValidatorSet::Multi(ref map) => {
				assert_eq!(map.len(), 3);
				assert!(map.contains_key(&Uint(U256::from(0))));
				assert!(map.contains_key(&Uint(U256::from(10))));
				assert!(map.contains_key(&Uint(U256::from(20))));
			},
			_ => assert!(false),
		}
	}
}
