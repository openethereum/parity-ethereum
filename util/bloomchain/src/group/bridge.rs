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
