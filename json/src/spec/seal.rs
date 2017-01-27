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

//! Spec seal deserialization.

use hash::*;
use uint::Uint;
use bytes::Bytes;

/// Ethereum seal.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Ethereum {
	/// Seal nonce.
	pub nonce: H64,
	/// Seal mix hash.
	#[serde(rename="mixHash")]
	pub mix_hash: H256,
}

/// AuthorityRound seal.
#[derive(Debug, PartialEq, Deserialize)]
pub struct AuthorityRoundSeal {
	/// Seal step.
	pub step: Uint,
	/// Seal signature.
	pub signature: H520,
}

/// Tendermint seal.
#[derive(Debug, PartialEq, Deserialize)]
pub struct TendermintSeal {
	/// Seal round.
	pub round: Uint,
	/// Proposal seal signature.
	pub proposal: H520,
	/// Proposal seal signature.
	pub precommits: Vec<H520>,
}

/// Seal variants.
#[derive(Debug, PartialEq, Deserialize)]
pub enum Seal {
	/// Ethereum seal.
	#[serde(rename="ethereum")]
	Ethereum(Ethereum),
	/// AuthorityRound seal.
	#[serde(rename="authorityRound")]
	AuthorityRound(AuthorityRoundSeal),
	/// Tendermint seal.
	#[serde(rename="tendermint")]
	Tendermint(TendermintSeal),
	/// Generic seal.
	#[serde(rename="generic")]
	Generic(Bytes),
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::Seal;

	#[test]
	fn seal_deserialization() {
		let s = r#"[{
			"ethereum": {
				"nonce": "0x0000000000000042",
				"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
			}
		},{
			"generic": "0xe011bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa"
		},{
			"authorityRound": {
				"step": "0x0",
				"signature": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
			}
		},{
			"tendermint": {
				"round": "0x0",
				"proposal": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
				"precommits": [
					"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
				]
			}
		}]"#;
		let _deserialized: Vec<Seal> = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
