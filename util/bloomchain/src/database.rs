use position::Position;
use Bloom;

/// Readonly `Bloom` database.
pub trait BloomDatabase {
	fn bloom_at(&self, position: &Position) -> Option<Bloom>;
}
