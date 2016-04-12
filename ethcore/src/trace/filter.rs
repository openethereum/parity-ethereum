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

use util::{Address, H2048, FixedHash};
use util::sha3::Hashable;
use basic_types::LogBloom;
use client::BlockId;

/// Traces filter.
pub struct Filter {
	/// Traces will be searched from this block.
	pub from_block: BlockId,

	/// Till this block.
	pub to_block: BlockId,

	/// From address. If empty, match all, if not, match one of the values.
	pub from_address: Vec<Address>,

	/// To address. If empty, match all, if not, match one of the values.
	pub to_address: Vec<Address>,
}

impl Filter {
	/// Returns combinations of each address.
	pub fn bloom_possibilities(&self) -> Vec<LogBloom> {
		let blooms = match self.from_address.is_empty() {
			true => vec![LogBloom::new()],
			false => self.from_address
				.iter()
				.map(|address| LogBloom::from_bloomed(&address.sha3()))
				.collect()
		};

		blooms
			.into_iter()
			.flat_map(|bloom| self.to_address
				.iter()
				.map(| address | bloom.with_bloomed(&address.sha3()))
				.collect::<Vec<_>>())
			.collect()
	}
}
