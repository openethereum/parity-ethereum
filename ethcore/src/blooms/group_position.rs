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

use bloomchain::group as bc;
use util::HeapSizeOf;

/// Represents BloomGroup position in database.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct GroupPosition {
	/// Bloom level.
	pub level: u8,
	/// Group index.
	pub index: u32,
}

impl From<bc::GroupPosition> for GroupPosition {
	fn from(p: bc::GroupPosition) -> Self {
		GroupPosition {
			level: p.level as u8,
			index: p.index as u32,
		}
	}
}

impl HeapSizeOf for GroupPosition {
	fn heap_size_of_children(&self) -> usize {
		0
	}
}
