//! Contract event specification.

use super::EventParam;

/// Contract event specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Event {
	/// Event name.
	pub name: String,
	/// Event input.
	pub inputs: Vec<EventParam>,
	/// If anonymous, event cannot be found using `from` filter.
	pub anonymous: bool,
}

impl Event {
	/// Returns all indexed params of the event.
	pub fn indexed_params(&self) -> Vec<EventParam> {
		self.inputs.iter()
			.filter(|p| p.indexed)
			.cloned()
			.collect()
	}
}
