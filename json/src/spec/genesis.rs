// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Spec genesis deserialization.

use bytes::Bytes;
use hash::{Address, H256};
use spec::Seal;
use uint::{self, Uint};

/// Spec genesis.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    /// Seal.
    pub seal: Seal,
    /// Difficulty.
    pub difficulty: Uint,
    /// Block author, defaults to 0.
    pub author: Option<Address>,
    /// Block timestamp, defaults to 0.
    pub timestamp: Option<Uint>,
    /// Parent hash, defaults to 0.
    pub parent_hash: Option<H256>,
    /// Gas limit.
    #[serde(deserialize_with = "uint::validate_non_zero")]
    pub gas_limit: Uint,
    /// Transactions root.
    pub transactions_root: Option<H256>,
    /// Receipts root.
    pub receipts_root: Option<H256>,
    /// State root.
    pub state_root: Option<H256>,
    /// Gas used.
    pub gas_used: Option<Uint>,
    /// Extra data.
    pub extra_data: Option<Bytes>,
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use ethereum_types::{H160, H256 as Eth256, H64 as Eth64, U256};
    use hash::{Address, H256, H64};
    use serde_json;
    use spec::{genesis::Genesis, Ethereum, Seal};
    use std::str::FromStr;
    use uint::Uint;

    #[test]
    fn genesis_deserialization() {
        let s = r#"{
			"difficulty": "0x400000000",
			"seal": {
				"ethereum": {
					"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
					"nonce": "0x00006d6f7264656e"
				}
			},
			"author": "0x1000000000000000000000000000000000000001",
			"timestamp": "0x07",
			"parentHash": "0x9000000000000000000000000000000000000000000000000000000000000000",
			"extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
			"gasLimit": "0x1388",
			"stateRoot": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"
		}"#;
        let deserialized: Genesis = serde_json::from_str(s).unwrap();
        assert_eq!(
            deserialized,
            Genesis {
                seal: Seal::Ethereum(Ethereum {
                    nonce: H64(Eth64::from("0x00006d6f7264656e")),
                    mix_hash: H256(Eth256::from(
                        "0x0000000000000000000000000000000000000000000000000000000000000000"
                    ))
                }),
                difficulty: Uint(U256::from(0x400000000u64)),
                author: Some(Address(H160::from(
                    "0x1000000000000000000000000000000000000001"
                ))),
                timestamp: Some(Uint(U256::from(0x07))),
                parent_hash: Some(H256(Eth256::from(
                    "0x9000000000000000000000000000000000000000000000000000000000000000"
                ))),
                gas_limit: Uint(U256::from(0x1388)),
                transactions_root: None,
                receipts_root: None,
                state_root: Some(H256(Eth256::from(
                    "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"
                ))),
                gas_used: None,
                extra_data: Some(
                    Bytes::from_str(
                        "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa"
                    )
                    .unwrap()
                ),
            }
        );
    }
}
