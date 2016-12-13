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

/// Validator lists.

mod simple_list;
mod contract;

pub use self::simple_list::SimpleList;
pub use self::contract::ValidatorContract;

use util::Address;

pub trait ValidatorSet {
	/// Checks if a given address is a validator.
	fn contains(&self, address: &Address) -> bool;
	/// Draws an validator nonce modulo number of validators.
	fn get(&self, nonce: usize) -> Address;
}
