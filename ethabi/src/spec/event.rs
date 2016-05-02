use super::{EventParam, ParamType};

#[derive(Debug, Clone)]
pub struct Event {
	pub name: String,
	pub inputs: Vec<EventParam>,
	pub anonymous: bool,
}

impl Event {
	pub fn indexed_params(&self) -> Vec<EventParam> {
		self.inputs.iter()
			.filter(|p| p.indexed)
			.cloned()
			.collect()
	}
}
