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

//! Spec params deserialization.

use uint::Uint;
use hash::H256;

/// Spec params.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Params {
	/// Account start nonce.
	#[serde(rename="accountStartNonce")]
	pub account_start_nonce: Uint,
	/// Maximum size of extra data.
	#[serde(rename="maximumExtraDataSize")]
	pub maximum_extra_data_size: Uint,
	/// Network id.
	#[serde(rename="networkID")]
	pub network_id: Uint,
	/// Name of the main ("eth") subprotocol.
	#[serde(rename="subprotocolName")]
	pub subprotocol_name: Option<String>,
	/// Minimum gas limit.
	#[serde(rename="minGasLimit")]
	pub min_gas_limit: Uint,
	/// Option fork block number to check.
	#[serde(rename="forkBlock")]
	pub fork_block: Option<Uint>,
	/// Expected fork block hash.
	#[serde(rename="forkCanonHash")]
	pub fork_hash: Option<H256>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::params::Params;

	#[test]
	fn params_deserialization() {
		let s = r#"{
			"homesteadTransition": "0x118c30",
			"maximumExtraDataSize": "0x20",
			"networkID" : "0x1",
			"subprotocolName" : "exp",
			"minGasLimit": "0x1388",
			"accountStartNonce": "0x00"
		}"#;

		let _deserialized: Params = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
