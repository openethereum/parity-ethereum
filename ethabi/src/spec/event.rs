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
	/// Returns names of all params.
	pub fn params_names(&self) -> Vec<String> {
		self.inputs.iter()
			.map(|p| p.name.clone())
			.collect()
	}

	/// Returns all params of the event.
	pub fn indexed_params(&self, indexed: bool) -> Vec<EventParam> {
		self.inputs.iter()
			.filter(|p| p.indexed == indexed)
			.cloned()
			.collect()
	}
}
