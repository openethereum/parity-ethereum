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

//! Spec deserialization.

use std::collections::BTreeMap;
use hash::Address;
use spec::account::Account;
use spec::params::Params;
use spec::genesis::Genesis;

/// Spec deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Spec {
	name: String,
	#[serde(rename="engineName")]
	engine_name: String, // TODO: consider making it an enum
	params: Params,
	genesis: Genesis,
	accounts: BTreeMap<Address, Account>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::spec::Spec;

	#[test]
	fn spec_deserialization() {
		let s = r#"{
	"name": "Morden",
	"engineName": "Ethash",
	"params": {
		"accountStartNonce": "0x0100000",
		"frontierCompatibilityModeLimit": "0x789b0",
		"maximumExtraDataSize": "0x20",
		"tieBreakingGas": false,
		"minGasLimit": "0x1388",
		"gasLimitBoundDivisor": "0x0400",
		"minimumDifficulty": "0x020000",
		"difficultyBoundDivisor": "0x0800",
		"durationLimit": "0x0d",
		"blockReward": "0x4563918244F40000",
		"registrar": "",
		"networkID" : "0x2"
	},
	"genesis": {
		"nonce": "0x00006d6f7264656e",
		"difficulty": "0x20000",
		"mixHash": "0x00000000000000000000000000000000000000647572616c65787365646c6578",
		"author": "0x0000000000000000000000000000000000000000",
		"timestamp": "0x00",
		"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"extraData": "0x",
		"gasLimit": "0x2fefd8"
	},
	"nodes": [
		"enode://b1217cbaa440e35ed471157123fe468e19e8b5ad5bedb4b1fdbcbdab6fb2f5ed3e95dd9c24a22a79fdb2352204cea207df27d92bfd21bfd41545e8b16f637499@104.44.138.37:30303"
	],
	"accounts": {
		"0000000000000000000000000000000000000001": { "balance": "1", "nonce": "1048576", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
		"0000000000000000000000000000000000000002": { "balance": "1", "nonce": "1048576", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
		"0000000000000000000000000000000000000003": { "balance": "1", "nonce": "1048576", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
		"0000000000000000000000000000000000000004": { "balance": "1", "nonce": "1048576", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
		"102e61f5d8f9bc71d0ad4a084df4e65e05ce0e1c": { "balance": "1606938044258990275541962092341162602522202993782792835301376", "nonce": "1048576" }
	}
		}"#;
		let _deserialized: Spec = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
