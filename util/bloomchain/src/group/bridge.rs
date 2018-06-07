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

use bloom::Bloom;
use config::Config;
use database::BloomDatabase;
use position::Position;
use group::position::Manager as PositionManager;
use super::BloomGroupDatabase;

/// Bridge between `BloomDatabase` and `BloomGroupDatabase`.
pub struct GroupDatabaseBridge<'a> {
	positioner: PositionManager,	
	db: &'a BloomGroupDatabase,
}

impl<'a> GroupDatabaseBridge<'a> {
	pub fn new(config: Config, db: &'a BloomGroupDatabase) -> Self {
		let positioner = PositionManager::new(config.elements_per_index);

		GroupDatabaseBridge {
			positioner: positioner,
			db: db,
		}
	}
}

impl<'a> BloomDatabase for GroupDatabaseBridge<'a> {
	fn bloom_at(&self, position: &Position) -> Option<Bloom> {
		let position = self.positioner.position(position);
		self.db.blooms_at(&position.group)
			.and_then(|group| group.blooms.into_iter().nth(position.number))
	}
}
