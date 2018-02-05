use position::Position;
use bloom::Bloom;

/// Readonly `Bloom` database.
pub trait BloomDatabase {
	fn bloom_at(&self, position: &Position) -> Option<Bloom>;
}
