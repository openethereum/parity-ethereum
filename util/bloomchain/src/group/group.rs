use bloom::Bloom;

/// Group of blooms that are in the same index.
#[derive(Debug, Clone)]
pub struct BloomGroup {
	pub blooms: Vec<Bloom>,
}

impl BloomGroup {
	pub fn new(size: usize) -> Self {
		let blooms = (0..size).into_iter().map(|_| Bloom::default()).collect();

		BloomGroup {
			blooms: blooms
		}
	}
}
