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

pub use self::version::Version;
pub use self::id::Id;
pub use self::params::Params;
pub use self::request::{Request, RequestBatchSlice, MethodCall, Notification};
pub use self::response::{Response, ResponseBatchSlice, Success, Failure};
pub use self::error::{ErrorCode, Error};
pub use serde_json::Value;

