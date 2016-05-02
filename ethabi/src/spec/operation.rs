use super::{Function, Event};

pub enum Operation {
	Constructor(Function),
	Function(Function),
	Event(Event),
}

impl Operation {
	pub fn function(&self) -> Option<&Function> {
		match *self {
			Operation::Function(ref f) => Some(f),
			_ => None
		}
	}

	pub fn event(&self) -> Option<&Event> {
		match *self {
			Operation::Event(ref e) => Some(e),
			_ => None
		}
	}
}
