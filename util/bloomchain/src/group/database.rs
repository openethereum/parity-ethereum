use group::{GroupPosition, BloomGroup};

/// Readonly `BloomGroup` database.
pub trait BloomGroupDatabase {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup>;
}
