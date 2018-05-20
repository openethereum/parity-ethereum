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

use super::{Position, GroupPosition};
use position::Position as BloomPosition;

pub struct Manager {
	index_size: usize
}

impl Manager {
	pub fn new(index_size: usize) -> Self {
		Manager {
			index_size: index_size
		}
	}

	pub fn group_position(&self, pos: &BloomPosition) -> GroupPosition {
		GroupPosition {
			level: pos.level,
			index: pos.index / self.index_size,
		}
	}

	pub fn position(&self, pos: &BloomPosition) -> Position {
		Position {
			group: self.group_position(pos),
			number: pos.index % self.index_size,	
		}
	}
}
