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

//! Transaction test deserialization.

use uint::Uint;
use bytes::Bytes;
use hash::Address;
use std::collections::BTreeMap;
use ethereum_types::H256;
use state::test::ForkSpec;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor, IgnoredAny};
use std::fmt;
use serde_json;

/// Transaction test deserialization.
#[derive(Debug, PartialEq)]
pub struct TransactionTest {
	/// Transaction rlp.
	pub rlp: Bytes,
	pub infos: BTreeMap<ForkSpec, TransactionTestInfos>,
}

/// Transaction test info for each chain spec.
#[derive(Debug, PartialEq, Deserialize)]
pub struct TransactionTestInfos {
	/// Transaction sender.
	pub sender: Option<Address>,
	/// Transaction hash
	pub hash: Option<H256>,
}

impl<'de> Deserialize<'de> for TransactionTest {
	fn deserialize<D>(deserializer: D) -> Result<TransactionTest, D::Error>
		where D: Deserializer<'de> {
		let (rlp, infos) = deserializer.deserialize_map(VisitorTransaction)?;
		Ok(TransactionTest { rlp, infos })
	}
}

struct VisitorTransaction;

impl<'de> Visitor<'de> for VisitorTransaction
{
	type Value = (Bytes, BTreeMap<ForkSpec, TransactionTestInfos>);

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("Transaction test visitor expect map")
	}

	fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
	where M: MapAccess<'de> {
		let mut map = BTreeMap::new();
		let mut rlp = None;

		loop {
			match access.next_key::<String>()? {
				Some(ref k) if k == "rlp" => {
					rlp = Some(access.next_value()?);
				},
				Some(ref k) => {
					if let Ok(fork_spec) = serde_json::from_str::<ForkSpec>(k) { 
						let v = access.next_value()?;
						map.insert(fork_spec, v);
					} else {
						// skip unknown
						access.next_value::<IgnoredAny>()?;
					}
				},
				None => break,
			};
		}

		Ok((rlp.expect("No rlp"), map))
	}
}


#[cfg(test)]
mod tests {
	use serde_json;
	use transaction::TransactionTest;

	#[test]
	fn transaction_deserialization() {
		let s = r#"{
        "Byzantium" : {
            "hash" : "2781a1444a7a4a646bf551f90913054dc47b2f3493d4a82a057445eb9e1c98cf",
            "sender" : "2fbffb0b9f709fd1fa4db9ff7342f2e6b3b2b7a6"
        },
        "Constantinople" : {
            "hash" : "2781a1444a7a4a646bf551f90913054dc47b2f3493d4a82a057445eb9e1c98cf",
            "sender" : "2fbffb0b9f709fd1fa4db9ff7342f2e6b3b2b7a6"
        },
        "EIP150" : {
            "hash" : "2781a1444a7a4a646bf551f90913054dc47b2f3493d4a82a057445eb9e1c98cf",
            "sender" : "2fbffb0b9f709fd1fa4db9ff7342f2e6b3b2b7a6"
        },
        "EIP158" : {
            "hash" : "2781a1444a7a4a646bf551f90913054dc47b2f3493d4a82a057445eb9e1c98cf",
            "sender" : "2fbffb0b9f709fd1fa4db9ff7342f2e6b3b2b7a6"
        },
        "Frontier" : {
            "hash" : "2781a1444a7a4a646bf551f90913054dc47b2f3493d4a82a057445eb9e1c98cf",
            "sender" : "2fbffb0b9f709fd1fa4db9ff7342f2e6b3b2b7a6"
        },
        "Homestead" : {
            "hash" : "2781a1444a7a4a646bf551f90913054dc47b2f3493d4a82a057445eb9e1c98cf",
            "sender" : "2fbffb0b9f709fd1fa4db9ff7342f2e6b3b2b7a6"
        },
        "_info" : {
            "comment" : "",
            "filledwith" : "cpp-1.3.0+commit.1829957d.Linux.g++",
            "lllcversion" : "Version: 0.4.18-develop.2017.10.11+commit.81f9f86c.Linux.g++",
            "source" : "src/TransactionTestsFiller/ttAddress/AddressLessThan20Prefixed0Filler.json",
            "sourceHash" : "c10a162dc48a3bc2a5f245c6c0aaede958ba6d76352907d777693e49cd621abe"
        },
        "rlp" : "0xf85f800182520894000000000000000000000000000b9331677e6ebf0a801ca098ff921201554726367d2be8c804a7ff89ccf285ebc57dff8ae4c44b9c19ac4aa01887321be575c8095f789dd4c743dfe42c1820f9231f98a962b210e3ac2452a3"
		}"#;
		let _deserialized: TransactionTest = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
