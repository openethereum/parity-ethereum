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

pub use self::version::Version;
pub use self::id::Id;
pub use self::params::Params;
pub use self::request::{Request, Call, MethodCall, Notification};
pub use self::response::{Response, Output, Success, Failure};
pub use self::error::{ErrorCode, Error};
pub use serde_json::Value;
pub use self::commander::{Commander, MethodCommand, NotificationCommand};
pub use self::request_handler::RequestHandler;

