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

//! State of all accounts in the system expressed in Plain Old Data.

use util::*;
use pod_account::*;
use ethjson;

/// State of all accounts in the system expressed in Plain Old Data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PodState (BTreeMap<Address, PodAccount>);

impl PodState {
	/// Contruct a new object from the `m`.
	pub fn new() -> PodState { Default::default() }

	/// Contruct a new object from the `m`.
	#[cfg(test)]
	pub fn from(m: BTreeMap<Address, PodAccount>) -> PodState { PodState(m) }

	/// Get the underlying map.
	pub fn get(&self) -> &BTreeMap<Address, PodAccount> { &self.0 }

	/// Get the root hash of the trie of the RLP of this.
	pub fn root(&self) -> H256 {
		sec_trie_root(self.0.iter().map(|(k, v)| (k.to_vec(), v.rlp())).collect())
	}

	/// Drain object to get the underlying map.
	#[cfg(test)]
	#[cfg(feature = "json-tests")]
	pub fn drain(self) -> BTreeMap<Address, PodAccount> { self.0 }
}

impl FromJson for PodState {
	/// Translate the JSON object into a hash map of account information ready for insertion into State.
	fn from_json(json: &Json) -> PodState {
		PodState(json.as_object().unwrap().iter().fold(BTreeMap::new(), |mut state, (address, acc)| {
			let balance = acc.find("balance").map(&U256::from_json);
			let nonce = acc.find("nonce").map(&U256::from_json);
			let storage = acc.find("storage").map(&BTreeMap::from_json);
			let code = acc.find("code").map(&Bytes::from_json);
			if balance.is_some() || nonce.is_some() || storage.is_some() || code.is_some() {
				state.insert(address_from_hex(address), PodAccount{
					balance: balance.unwrap_or_else(U256::zero),
					nonce: nonce.unwrap_or_else(U256::zero),
					storage: storage.unwrap_or_else(BTreeMap::new),
					code: code.unwrap_or_else(Vec::new)
				});
			}
			state
		}))
	}
}

impl From<ethjson::blockchain::State> for PodState {
	fn from(s: ethjson::blockchain::State) -> PodState {
		PodState(s.0.into_iter().fold(BTreeMap::new(), |mut acc, (key, value)| {
			acc.insert(key.into(), PodAccount::from(value));
			acc
		}))
	}
}

impl fmt::Display for PodState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (add, acc) in &self.0 {
			try!(writeln!(f, "{} => {}", add, acc));
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	extern crate rustc_serialize;

	use super::*;
	use rustc_serialize::*;
	use util::from_json::FromJson;
	use util::hash::*;

	#[test]
	fn it_serializes_form_json() {
		let pod_state = PodState::from_json(&json::Json::from_str(
r#"
	{
		"0000000000000000000000000000000000000000": {
			"balance": "1000",
			"nonce": "100",
			"storage": {},
			"code" : []
		}
	}
"#
		).unwrap());

		assert!(pod_state.get().get(&ZERO_ADDRESS).is_some());
	}
}
