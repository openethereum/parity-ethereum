#[derive(Debug)]
pub struct Options {
	pub max_count: usize,
	pub max_per_sender: usize,
	pub max_mem_usage: usize,
}

impl Default for Options {
	fn default() -> Self {
		Options {
			max_count: 1024,
			max_per_sender: 16,
			max_mem_usage: 8 * 1024 * 1024,
		}
	}
}
