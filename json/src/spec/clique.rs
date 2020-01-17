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

//! Clique params deserialization.

use std::num::NonZeroU64;
use serde::Deserialize;

/// Clique params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct CliqueParams {
	/// period as defined in EIP 225
	pub period: Option<u64>,
	/// epoch length as defined in EIP 225
	pub epoch: Option<NonZeroU64>
}

/// Clique engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Clique {
	/// CliqueEngine params
	pub params: CliqueParams,
}

#[cfg(test)]
mod tests {
	use super::{Clique, NonZeroU64};

	#[test]
	fn clique_deserialization() {
		let s = r#"{
			"params": {
				"period": 5,
				"epoch": 30000
			}
		}"#;

		let deserialized: Clique = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.params.period, Some(5u64));
		assert_eq!(deserialized.params.epoch, NonZeroU64::new(30000));
	}
}
