//! ### Transport agnostic jsonrpc library.
//! 
//! Right now it supports only server side handling requests.
//! 
//! ```rust
//! extern crate jsonrpc;
//! use jsonrpc::*;
//!
//! fn main() {
//! 	let mut io = IoHandler::new();
//! 	struct SayHello;
//! 	impl MethodCommand for SayHello {
//! 		fn execute(&mut self, _params: Option<Params>) -> Result<Value, Error> {
//! 			Ok(Value::String("hello".to_string()))
//! 		}
//! 	}
//!
//! 	io.add_method("say_hello", SayHello);
//!
//! 	let request = r#"{"jsonrpc": "2.0", "method": "say_hello", "params": [42, 23], "id": 1}"#;
//! 	let response = r#"{"jsonrpc":"2.0","result":"hello","id":1}"#;
//!
//! 	assert_eq!(io.handle_request(request), Some(response.to_string()));
//! }
//! ```
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

macro_rules! ok {
	($expr:expr) => (match $expr {
		Result::Err(_) => (),
		res => return res
	})
}

pub mod version;
pub mod id;
pub mod params;
pub mod request;
pub mod response;
pub mod error;
pub mod commander;
pub mod request_handler;
pub mod io;

pub use self::version::Version;
pub use self::id::Id;
pub use self::params::Params;
pub use self::request::{Request, Call, MethodCall, Notification};
pub use self::response::{Response, Output, Success, Failure};
pub use self::error::{ErrorCode, Error};
pub use serde_json::Value;
pub use self::commander::{Commander, MethodCommand, NotificationCommand};
pub use self::request_handler::RequestHandler;
pub use self::io::IoHandler;

