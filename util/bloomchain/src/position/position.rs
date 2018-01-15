/// Uniquely identifies bloom position.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Position {
	/// Bloom level.
	pub level: usize,
	/// Index of the bloom.
	pub index: usize,
}
