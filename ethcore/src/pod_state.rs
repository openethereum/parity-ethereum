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

impl From<ethjson::blockchain::State> for PodState {
	fn from(s: ethjson::blockchain::State) -> PodState {
		let state = s.into_iter().map(|(addr, acc)| (addr.into(), PodAccount::from(acc))).collect();
		PodState(state)
	}
}

impl From<ethjson::spec::State> for PodState {
	fn from(s: ethjson::spec::State) -> PodState {
		let state: BTreeMap<_,_> = s.into_iter()
			.filter(|pair| !pair.1.is_empty())
			.map(|(addr, acc)| (addr.into(), PodAccount::from(acc)))
			.collect();
		PodState(state)
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

