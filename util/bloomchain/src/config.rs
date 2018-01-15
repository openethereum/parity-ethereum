/// `BloomChain` configuration.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Config {
	/// Number of levels.
	pub levels: usize,
	/// Number of elements in a single index.
	pub elements_per_index: usize,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			levels: 3,
			elements_per_index: 16,
		}
	}
}
