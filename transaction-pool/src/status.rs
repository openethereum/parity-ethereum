#[derive(Debug, PartialEq, Eq)]
pub struct LightStatus {
	pub mem_usage: usize,
	pub count: usize,
	pub senders: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Status {
	pub stalled: usize,
	pub pending: usize,
	pub future: usize,
	pub senders: usize,
}
