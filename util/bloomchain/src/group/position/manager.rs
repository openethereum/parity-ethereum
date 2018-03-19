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
