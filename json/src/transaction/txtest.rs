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

use std::collections::BTreeMap;
use bytes::Bytes;
use hash::Address;
use hash::H256;
use spec::ForkSpec;

/// Transaction test deserialization.
#[derive(Debug, Deserialize)]
pub struct TransactionTest {
	pub rlp: Bytes,
	pub _info: ::serde::de::IgnoredAny,
	#[serde(flatten)]
	pub post_state: BTreeMap<ForkSpec, PostState>,
}

/// TransactionTest post state.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostState {
	/// Transaction sender.
	pub sender: Option<Address>,
	/// Transaction hash.
	pub hash: Option<H256>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use transaction::TransactionTest;

	#[test]
	fn transaction_deserialization() {
		let s = r#"{
			"Byzantium" : {
				"hash" : "4782cb5edcaeda1f0aef204b161214f124cefade9e146245183abbb9ca01bca5",
				"sender" : "2ea991808ba979ba103147edfd72304ebd95c028"
			},
			"Constantinople" : {
				"hash" : "4782cb5edcaeda1f0aef204b161214f124cefade9e146245183abbb9ca01bca5",
				"sender" : "2ea991808ba979ba103147edfd72304ebd95c028"
			},
			"EIP150" : {
			},
			"EIP158" : {
				"hash" : "4782cb5edcaeda1f0aef204b161214f124cefade9e146245183abbb9ca01bca5",
				"sender" : "2ea991808ba979ba103147edfd72304ebd95c028"
			},
			"Frontier" : {
			},
			"Homestead" : {
			},
			"_info" : {
				"comment" : "",
				"filledwith" : "cpp-1.3.0+commit.1829957d.Linux.g++",
				"lllcversion" : "Version: 0.4.18-develop.2017.10.11+commit.81f9f86c.Linux.g++",
				"source" : "src/TransactionTestsFiller/ttVValue/V_equals37Filler.json",
				"sourceHash" : "89ef69312d4c0b4e3742da501263d23d2a64f180258ac93940997ac6a17b9b19"
			},
			"rlp" : "0xf865808698852840a46f82d6d894095e7baea6a6c7c4c2dfeb977efac326af552d87808025a098ff921201554726367d2be8c804a7ff89ccf285ebc57dff8ae4c44b9c19ac4aa01887321be575c8095f789dd4c743dfe42c1820f9231f98a962b210e3ac2452a3"
		}"#;

		let _deserialized: TransactionTest = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
