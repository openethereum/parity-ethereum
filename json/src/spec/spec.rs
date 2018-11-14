// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::io::Read;
use serde_json;
use serde_json::Error;
use spec::{Params, Genesis, Engine, State, HardcodedSync};

/// Fork spec definition
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum ForkSpec {
	EIP150,
	EIP158,
	Frontier,
	Homestead,
	Byzantium,
	Constantinople,
	EIP158ToByzantiumAt5,
	FrontierToHomesteadAt5,
	HomesteadToDaoAt5,
	HomesteadToEIP150At5,
}

/// Spec deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spec {
	/// Spec name.
	pub name: String,
	/// Special fork name.
	pub data_dir: Option<String>,
	/// Engine.
	pub engine: Engine,
	/// Spec params.
	pub params: Params,
	/// Genesis header.
	pub genesis: Genesis,
	/// Genesis state.
	pub accounts: State,
	/// Boot nodes.
	pub nodes: Option<Vec<String>>,
	/// Hardcoded synchronization for the light client.
	pub hardcoded_sync: Option<HardcodedSync>,
}

impl Spec {
	/// Loads test from json.
	pub fn load<R>(reader: R) -> Result<Self, Error> where R: Read {
		serde_json::from_reader(reader)
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::spec::Spec;

	#[test]
	fn spec_deserialization() {
		let s = r#"{
	"name": "Morden",
	"dataDir": "morden",
	"engine": {
		"Ethash": {
			"params": {
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"durationLimit": "0x0d",
				"homesteadTransition" : "0x",
				"daoHardforkTransition": "0xffffffffffffffff",
				"daoHardforkBeneficiary": "0x0000000000000000000000000000000000000000",
				"daoHardforkAccounts": []
			}
		}
	},
	"params": {
		"accountStartNonce": "0x0100000",
		"homesteadTransition": "0x789b0",
		"maximumExtraDataSize": "0x20",
		"minGasLimit": "0x1388",
		"networkID" : "0x2",
		"forkBlock": "0xffffffffffffffff",
		"forkCanonHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"gasLimitBoundDivisor": "0x20"
	},
	"genesis": {
		"seal": {
			"ethereum": {
				"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"nonce": "0x00006d6f7264656e"
			}
		},
		"difficulty": "0x20000",
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
	},
	"hardcodedSync": {
		"header": "f901f9a0d405da4e66f1445d455195229624e133f5baafe72b5cf7b3c36c12c8146e98b7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a05fb2b4bfdef7b314451cb138a534d225c922fc0e5fbe25e451142732c3e25c25a088d2ec6b9860aae1a2c3b299f72b6a5d70d7f7ba4722c78f2c49ba96273c2158a007c6fdfa8eea7e86b81f5b0fc0f78f90cc19f4aa60d323151e0cac660199e9a1b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302008003832fefba82524d84568e932a80a0a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd88ab4e252a7e8c2a23",
		"totalDifficulty": "0x400000000",
		"CHTs": [
			"0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
			"0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"
		]
	}
		}"#;
		let _deserialized: Spec = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
