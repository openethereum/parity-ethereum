use std::collections::HashMap;
use bloomchain::{Position, Bloom, BloomDatabase};
use bloomchain::group::{GroupPosition, BloomGroup, BloomGroupDatabase};

#[derive(Default)]
pub struct BloomMemoryDatabase {
	mem: HashMap<Position, Bloom>,
}

impl BloomMemoryDatabase {
	#[allow(dead_code)]
	pub fn insert_blooms(&mut self, blooms: HashMap<Position, Bloom>) {
		self.mem.extend(blooms);
	}
}

impl BloomDatabase for BloomMemoryDatabase {
	fn bloom_at(&self, position: &Position) -> Option<Bloom> {
		self.mem.get(position).cloned()
	}
}

#[derive(Default)]
pub struct BloomGroupMemoryDatabase {
	mem: HashMap<GroupPosition, BloomGroup>,
}

impl BloomGroupMemoryDatabase {
	#[allow(dead_code)]
	pub fn insert_blooms(&mut self, groups: HashMap<GroupPosition, BloomGroup>) {
		self.mem.extend(groups);
	}
}

impl BloomGroupDatabase for BloomGroupMemoryDatabase {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup> {
		self.mem.get(position).cloned()
	}
}
