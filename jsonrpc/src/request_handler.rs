//! jsonrpc server request handler
use super::*;

pub struct RequestHandler {
	commander: Commander
}

impl RequestHandler {
	pub fn new() -> Self {
		RequestHandler {
			commander: Commander::new()
		}
	}

	#[inline]
	pub fn add_method<C>(&mut self, name: &str, command: C) where C: MethodCommand + 'static {
		self.commander.add_method(name, command)
	}

	#[inline]
	pub fn add_notification<C>(&mut self, name: &str, command: C) where C: NotificationCommand + 'static {
		self.commander.add_notification(name, command)
	}

	pub fn handle_request(&mut self, request: Request) -> Option<Response> {
		match request {
			Request::Single(call) => self.handle_call(call).map(Response::Single),
			Request::Batch(calls) => {
				let outs: Vec<Output> = calls.into_iter().filter_map(|call| self.handle_call(call)).collect();
				match outs.len() {
					0 => None,
					_ => Some(Response::Batch(outs))
				}
			}
		}
	}

	fn handle_call(&mut self, call: Call) -> Option<Output> {
		match call {
			Call::MethodCall(method) => Some(self.handle_method_call(method)),
			Call::Notification(notification) => {
				self.handle_notification(notification);
				None
			},
			Call::Invalid(_) => Some(Output::Failure(Failure {
				id: Id::Null,
				jsonrpc: Version::V2,
				error: Error::new(ErrorCode::InvalidRequest)
			}))
		}
	}

	fn handle_method_call(&mut self, method: MethodCall) -> Output {
		match self.commander.execute_method(method.method, method.params) {
			Ok(result) => Output::Success(Success {
				id: method.id,
				jsonrpc: method.jsonrpc,
				result: result
			}),
			Err(error) => Output::Failure(Failure {
				id: method.id,
				jsonrpc: method.jsonrpc,
				error: error
			})
		}
	}

	fn handle_notification(&mut self, notification: Notification) {
		self.commander.execute_notification(notification.method, notification.params)
	}
}
