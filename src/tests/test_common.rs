pub use common::*;

macro_rules! test {
	($name: expr) => {
		assert!(do_json_test(include_bytes!(concat!("../../res/ethereum/tests/", $name, ".json"))).is_empty());
	}
}

#[macro_export]
macro_rules! declare_test {
	(ignore => $id: ident, $name: expr) => {
		#[ignore]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	};
	(heavy => $id: ident, $name: expr) => {
		#[cfg(feature = "test-heavy")]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	};
	($id: ident, $name: expr) => {
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	}
}
