pub use common::*;

#[macro_export]
macro_rules! declare_test {
	($id: ident, $name: expr) => {
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			assert!(do_json_test(include_bytes!(concat!("../../res/ethereum/tests/", $name, ".json"))).is_empty());
		}
	};
}

#[macro_export]
macro_rules! declare_test_ignore {
	($id: ident, $name: expr) => {
		#[test]
		#[ignore]
		#[allow(non_snake_case)]
		fn $id() {
			assert!(do_json_test(include_bytes!(concat!("../../res/ethereum/tests/", $name, ".json"))).is_empty());
		}
	};
}
