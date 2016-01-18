//! jsonrpc errors
use serde::{Serialize, Serializer};
use super::Value;

#[derive(Debug, PartialEq)]
pub enum ErrorCode {
	/// Invalid JSON was received by the server.
	/// An error occurred on the server while parsing the JSON text.
	ParseError,
	/// The JSON sent is not a valid Request object.
	InvalidRequest,
	/// The method does not exist / is not available.
	MethodNotFound,
	/// Invalid method parameter(s).
	InvalidParams,
	/// Internal JSON-RPC error.
	InternalError,
	/// Reserved for implementation-defined server-errors.
	ServerError(i64)
}

impl ErrorCode {
	pub fn code(&self) -> i64 {
		match *self {
			ErrorCode::ParseError => -32700,
			ErrorCode::InvalidRequest => -32600,
			ErrorCode::MethodNotFound => -32601,
			ErrorCode::InvalidParams => -32602,
			ErrorCode::InternalError => -32603,
			ErrorCode::ServerError(code) => code
		}
	}

	pub fn description(&self) -> String {
		let desc = match *self {
			ErrorCode::ParseError => "Parse error.",
			ErrorCode::InvalidRequest => "Invalid request.",
			ErrorCode::MethodNotFound => "Method not found.",
			ErrorCode::InvalidParams => "Invalid params.",
			ErrorCode::InternalError => "Internal error.",
			ErrorCode::ServerError(_) => "Server error.",
		};
		desc.to_string()
	}
}

impl Serialize for ErrorCode {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		serializer.visit_i64(self.code())
	}
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Error {
	code: ErrorCode,
	message: String,
	data: Option<Value>
}

impl Error {
	pub fn new(code: ErrorCode) -> Error {
		Error {
			message: code.description(),
			code: code,
			data: None
		}
	}
}
