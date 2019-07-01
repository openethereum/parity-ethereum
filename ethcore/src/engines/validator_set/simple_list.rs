// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

/// Preconfigured validator list.

use parity_util_mem::MallocSizeOf;
use ethereum_types::{H256, Address};

use machine::{AuxiliaryData, Call, Machine};
use types::BlockNumber;
use types::header::Header;
use super::ValidatorSet;

/// Validator set containing a known set of addresses.
#[derive(Clone, Debug, PartialEq, Eq, Default, MallocSizeOf)]
pub struct SimpleList {
	validators: Vec<Address>,
}

impl SimpleList {
	/// Create a new `SimpleList`.
	pub fn new(validators: Vec<Address>) -> Self {
		let validator_count = validators.len();
		if validator_count == 1 {
			warn!(target: "engine", "Running AuRa with a single validator implies instant finality. Use a database?");
		} else if validator_count != 0 && validator_count % 2 == 0 {
			warn!(target: "engine", "Running AuRa with an even number of validators ({}) is not recommended (risk of network split).", validator_count);
		}
		SimpleList { validators }
	}

	/// Convert into inner representation.
	pub fn into_inner(self) -> Vec<Address> {
		self.validators
	}
}

impl ::std::ops::Deref for SimpleList {
	type Target = [Address];

	fn deref(&self) -> &[Address] { &self.validators }
}

impl From<Vec<Address>> for SimpleList {
	fn from(validators: Vec<Address>) -> Self {
		SimpleList::new(validators)
	}
}

impl ValidatorSet for SimpleList {
	fn default_caller(&self, _block_id: ::types::ids::BlockId) -> Box<Call> {
		Box::new(|_, _| Err("Simple list doesn't require calls.".into()))
	}

	fn is_epoch_end(&self, first: bool, _chain_head: &Header) -> Option<Vec<u8>> {
		match first {
			true => Some(Vec::new()), // allow transition to fixed list, and instantly
			false => None,
		}
	}

	fn signals_epoch_end(&self, _: bool, _: &Header, _: AuxiliaryData)
		-> ::engines::EpochChange
	{
		::engines::EpochChange::No
	}

	fn epoch_set(&self, _first: bool, _: &Machine, _: BlockNumber, _: &[u8]) -> Result<(SimpleList, Option<H256>), ::error::Error> {
		Ok((self.clone(), None))
	}

	fn contains_with_caller(&self, _bh: &H256, address: &Address, _: &Call) -> bool {
		self.validators.contains(address)
	}

	fn get_with_caller(&self, _bh: &H256, nonce: usize, _: &Call) -> Address {
		let validator_n = self.validators.len();

		if validator_n == 0 {
			panic!("Cannot operate with an empty validator set.");
		}

		self.validators.get(nonce % validator_n).expect("There are validator_n authorities; taking number modulo validator_n gives number in validator_n range; qed").clone()
	}

	fn count_with_caller(&self, _bh: &H256, _: &Call) -> usize {
		self.validators.len()
	}
}

impl AsRef<dyn ValidatorSet> for SimpleList {
    fn as_ref(&self) -> &dyn ValidatorSet {
        self
    }
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use ethereum_types::Address;
	use super::super::ValidatorSet;
	use super::SimpleList;

	#[test]
	fn simple_list() {
		let a1 = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let a2 = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let list = SimpleList::new(vec![a1.clone(), a2.clone()]);
		assert!(list.contains(&Default::default(), &a1));
		assert_eq!(list.get(&Default::default(), 0), a1);
		assert_eq!(list.get(&Default::default(), 1), a2);
		assert_eq!(list.get(&Default::default(), 2), a1);
	}
}
