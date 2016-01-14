use standard::*;

#[macro_export]
macro_rules! xjson {
	( $x:expr ) => {
		FromJson::from_json($x)
	}
}

pub trait FromJson {
	fn from_json(json: &Json) -> Self;
}
