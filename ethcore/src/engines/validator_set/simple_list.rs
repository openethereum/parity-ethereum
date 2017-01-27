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

/// Preconfigured validator list.

use util::Address;
use super::ValidatorSet;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct SimpleList {
	validators: Vec<Address>,
	validator_n: usize,
}

impl SimpleList {
	pub fn new(validators: Vec<Address>) -> Self {
		SimpleList {
			validator_n: validators.len(),
			validators: validators,
		}
	}
}

impl ValidatorSet for SimpleList {
	fn contains(&self, address: &Address) -> bool {
		self.validators.contains(address)
	}

	fn get(&self, nonce: usize) -> Address {
		self.validators.get(nonce % self.validator_n).expect("There are validator_n authorities; taking number modulo validator_n gives number in validator_n range; qed").clone()
	}

	fn count(&self) -> usize {
		self.validator_n
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::Address;
	use super::super::ValidatorSet;
	use super::SimpleList;

	#[test]
	fn simple_list() {
		let a1 = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let a2 = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let list = SimpleList::new(vec![a1.clone(), a2.clone()]);
		assert!(list.contains(&a1));
		assert_eq!(list.get(0), a1);
		assert_eq!(list.get(1), a2);
		assert_eq!(list.get(2), a1);
	}
}
