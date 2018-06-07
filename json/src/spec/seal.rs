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
	use hash::*;
	use bytes::Bytes;
	use uint::Uint;
	use ethereum_types::{U256, H64 as Eth64, H256 as Eth256, H520 as Eth520};
	use spec::{Ethereum, AuthorityRoundSeal, TendermintSeal, Seal};

	#[test]
	fn seal_deserialization() {
		let s = r#"[{
			"ethereum": {
				"nonce": "0x0000000000000042",
				"mixHash": "0x1000000000000000000000000000000000000000000000000000000000000001"
			}
		},{
			"generic": "0xe011bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa"
		},{
			"authorityRound": {
				"step": "0x0",
				"signature": "0x2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002"
			}
		},{
			"tendermint": {
				"round": "0x3",
				"proposal": "0x3000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003",
				"precommits": [
					"0x4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004"
				]
			}
		}]"#;

		let deserialized: Vec<Seal> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.len(), 4);

		// [0]
		assert_eq!(deserialized[0], Seal::Ethereum(Ethereum {
			nonce: H64(Eth64::from("0x0000000000000042")),
			mix_hash: H256(Eth256::from("0x1000000000000000000000000000000000000000000000000000000000000001"))
		}));

		// [1]
		assert_eq!(deserialized[1], Seal::Generic(Bytes::new(vec![
			0xe0, 0x11, 0xbb, 0xe8, 0xdb, 0x4e, 0x34, 0x7b, 0x4e, 0x8c, 0x93, 0x7c, 0x1c, 0x83, 0x70, 0xe4,
			0xb5, 0xed, 0x33, 0xad, 0xb3, 0xdb, 0x69, 0xcb, 0xdb, 0x7a, 0x38, 0xe1, 0xe5, 0x0b, 0x1b, 0x82, 0xfa])));

		// [2]
		assert_eq!(deserialized[2], Seal::AuthorityRound(AuthorityRoundSeal {
			step: Uint(U256::from(0x0)),
			signature: H520(Eth520::from("0x2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002"))
		}));

		// [3]
		assert_eq!(deserialized[3], Seal::Tendermint(TendermintSeal {
			round: Uint(U256::from(0x3)),
			proposal: H520(Eth520::from("0x3000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003")),
			precommits: vec![H520(Eth520::from("0x4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004"))]
		}));
	}
}
