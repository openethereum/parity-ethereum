//! jsonrpc io
use serde_json;
use super::*;

/// Should be used to handle jsonrpc io.
/// 
/// ```rust
/// extern crate jsonrpc;
/// use jsonrpc::*;
///
/// fn main() {
/// 	let mut io = IoHandler::new();
/// 	struct SayHello;
/// 	impl MethodCommand for SayHello {
/// 		fn execute(&mut self, _params: Option<Params>) -> Result<Value, Error> {
/// 			Ok(Value::String("hello".to_string()))
/// 		}
/// 	}
///
/// 	io.add_method("say_hello", SayHello);
///
/// 	let request = r#"{"jsonrpc": "2.0", "method": "say_hello", "params": [42, 23], "id": 1}"#;
/// 	let response = r#"{"jsonrpc":"2.0","result":"hello","id":1}"#;
///
/// 	assert_eq!(io.handle_request(request), Some(response.to_string()));
/// }
/// ```
pub struct IoHandler {
	request_handler: RequestHandler
}

fn read_request<'a>(request_str: &'a str) -> Result<Request, Error> {
	serde_json::from_str(request_str).map_err(|_| Error::new(ErrorCode::ParseError))
}

fn write_response(response: Response) -> String {
	// this should never fail
	serde_json::to_string(&response).unwrap()
}

impl IoHandler {
	pub fn new() -> Self {
		IoHandler {
			request_handler: RequestHandler::new()
		}
	}

	#[inline]
	pub fn add_method<C>(&mut self, name: &str, command: C) where C: MethodCommand + 'static {
		self.request_handler.add_method(name, command)
	}

	#[inline]
	pub fn add_notification<C>(&mut self, name: &str, command: C) where C: NotificationCommand + 'static {
		self.request_handler.add_notification(name, command)
	}

	pub fn handle_request<'a>(&mut self, request_str: &'a str) -> Option<String> {
		match read_request(request_str) {
			Ok(request) => self.request_handler.handle_request(request).map(write_response),
			Err(error) => Some(write_response(Response::Single(Output::Failure(Failure {
				id: Id::Null,
				jsonrpc: Version::V2,
				error: error
			}))))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::super::*;

	#[test]
	fn test_io_handler() {
		let mut io = IoHandler::new();
		
		struct SayHello;
		impl MethodCommand for SayHello {
			fn execute(&mut self, _params: Option<Params>) -> Result<Value, Error> {
				Ok(Value::String("hello".to_string()))
			}
		}

		io.add_method("say_hello", SayHello);
		
		let request = r#"{"jsonrpc": "2.0", "method": "say_hello", "params": [42, 23], "id": 1}"#;
		let response = r#"{"jsonrpc":"2.0","result":"hello","id":1}"#;

		assert_eq!(io.handle_request(request), Some(response.to_string()));
	}
}
