//! Coversion from json.

use standard::*;

#[macro_export]
macro_rules! xjson {
	( $x:expr ) => {
		FromJson::from_json($x)
	}
}

/// Trait allowing conversion from a JSON value.
pub trait FromJson {
	/// Convert a JSON value to an instance of this type.
	fn from_json(json: &Json) -> Self;
}
