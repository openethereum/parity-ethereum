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

//! Represents bloom index in cache

/// Represents bloom index in cache
/// 
/// On cache level 0, every block bloom is represented by different index.
/// On higher cache levels, multiple block blooms are represented by one
/// index. Their `BloomIndex` can be created from block number and given level.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct BloomIndex {
	/// Bloom level
	pub level: u8,
	///  Filter Index
	pub index: usize,
}

impl BloomIndex {
	/// Default constructor for `BloomIndex`
	pub fn new(level: u8, index: usize) -> BloomIndex {
		BloomIndex {
			level: level,
			index: index,
		}
	}
}
