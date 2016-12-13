// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
}
