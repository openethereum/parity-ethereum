pub use common::*;

pub fn clean(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" {
		&s[2..]
	} else {
		s
	}
}

pub fn bytes_from_json(json: &Json) -> Bytes {
	let s = json.as_string().unwrap();
	if s.len() % 2 == 1 {
		FromHex::from_hex(&("0".to_string() + &(clean(s).to_string()))[..]).unwrap_or(vec![])
	} else {
		FromHex::from_hex(clean(s)).unwrap_or(vec![])
	}
}

pub fn address_from_json(json: &Json) -> Address {
	address_from_str(json.as_string().unwrap())
}

pub fn address_from_str<'a>(s: &'a str) -> Address {
	if s.len() % 2 == 1 {
		address_from_hex(&("0".to_string() + &(clean(s).to_string()))[..])
	} else {
		address_from_hex(clean(s))
	}
}

pub fn u256_from_json(json: &Json) -> U256 {
	u256_from_str(json.as_string().unwrap())
}

pub fn u256_from_str<'a>(s: &'a str) -> U256 {
	if s.len() >= 2 && &s[0..2] == "0x" {
		// hex
		U256::from_str(&s[2..]).unwrap()
	}
	else {
		// dec
		U256::from_dec_str(s).unwrap()
	}
}

#[macro_export]
macro_rules! declare_test {
	($id: ident, $name: expr) => {
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			assert!(do_json_test(include_bytes!(concat!("../../res/ethereum/tests/", $name, ".json"))).len() == 0);
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
			assert!(do_json_test(include_bytes!(concat!("../../res/ethereum/tests/", $name, ".json"))).len() == 0);
		}
	};
}
