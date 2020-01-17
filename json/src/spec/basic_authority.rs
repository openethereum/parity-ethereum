// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Authority params deserialization.

use crate::uint::Uint;
use super::ValidatorSet;
use serde::Deserialize;

/// Authority params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct BasicAuthorityParams {
	/// Block duration.
	pub duration_limit: Uint,
	/// Valid authorities
	pub validators: ValidatorSet,
}

/// Authority engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicAuthority {
	/// Ethash params.
	pub params: BasicAuthorityParams,
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use super::{BasicAuthority, Uint};
	use ethereum_types::{U256, H160};
	use crate::{hash::Address, spec::validator_set::ValidatorSet};

	#[test]
	fn basic_authority_deserialization() {
		let s = r#"{
			"params": {
				"durationLimit": "0x0d",
				"validators" : {
					"list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
				}
			}
		}"#;

		let deserialized: BasicAuthority = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized.params.duration_limit, Uint(U256::from(0x0d)));
		let vs = ValidatorSet::List(vec![Address(H160::from_str("c6d9d2cd449a754c494264e1809c50e34d64562b").unwrap())]);
		assert_eq!(deserialized.params.validators, vs);
	}
}
