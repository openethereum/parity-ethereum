use super::{Operation, Function, Event};

pub struct Interface(Vec<Operation>);

impl Interface {
	pub fn function(&self, name: String) -> Option<Function> { 
		self.0.iter()
			.filter_map(Operation::function)
			.find(|f| f.name == name)
			.cloned()
	}

	pub fn event(&self, name: String) -> Option<Event> {
		self.0.iter()
			.filter_map(Operation::event)
			.find(|e| e.name == name)
			.cloned()
	}
}

