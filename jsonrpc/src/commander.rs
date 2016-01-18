//! method and notification commands executor

use std::collections::HashMap;
use super::{Params, Value, Error, ErrorCode};

/// Should be used to handle single method call.
pub trait MethodCommand {
	fn execute(&mut self, params: Option<Params>) -> Result<Value, Error>;
}

/// Should be used to handle single notification.
pub trait NotificationCommand {
	fn execute(&mut self, params: Option<Params>);
}

/// Commands executor.
pub struct Commander {
	methods: HashMap<String, Box<MethodCommand>>,
	notifications: HashMap<String, Box<NotificationCommand>>
}

impl Commander {
	pub fn new() -> Self {
		Commander {
			methods: HashMap::new(),
			notifications: HashMap::new()
		}
	}

	pub fn add_method<C>(&mut self, name: &str, command: C) where C: MethodCommand + 'static {
		self.methods.insert(name.to_string(), Box::new(command));
	}

	pub fn add_notification<C>(&mut self, name: &str, command: C) where C: NotificationCommand + 'static {
		self.notifications.insert(name.to_string(), Box::new(command));
	}

	pub fn execute_method(&mut self, name: String, params: Option<Params>) -> Result<Value, Error> {
		match self.methods.get_mut(&name) {
			Some(command) => command.execute(params),
			None => Err(Error::new(ErrorCode::MethodNotFound))
		}
	}

	pub fn execute_notification(&mut self, name: String, params: Option<Params>) {
		if let Some(command) = self.notifications.get_mut(&name) {
			command.execute(params)
		}
	}
}
