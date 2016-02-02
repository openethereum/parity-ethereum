//! Coversion from json.

use standard::*;

#[macro_export]
macro_rules! xjson {
	( $x:expr ) => {
		FromJson::from_json($x)
	}
}

/// TODO [Gav Wood] Please document me
pub trait FromJson {
	/// TODO [Gav Wood] Please document me
	fn from_json(json: &Json) -> Self;
}
