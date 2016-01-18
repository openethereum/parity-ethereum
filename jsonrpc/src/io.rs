//! jsonrpc io
use serde_json;
use super::*;

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
	pub fn add_method<C>(&mut self, name: String, command: C) where C: MethodCommand + 'static {
		self.request_handler.add_method(name, command)
	}

	#[inline]
	pub fn add_notification<C>(&mut self, name: String, command: C) where C: NotificationCommand + 'static {
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
