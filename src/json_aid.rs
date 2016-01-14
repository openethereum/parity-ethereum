use common::*;

pub fn clean(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" {
		&s[2..]
	} else {
		s
	}
}

fn u256_from_str(s: &str) -> U256 {
	if s.len() >= 2 && &s[0..2] == "0x" {
		U256::from_str(&s[2..]).unwrap_or(U256::from(0))
	} else {
		U256::from_dec_str(s).unwrap_or(U256::from(0))
	}
}

impl FromJson for Bytes {
	fn from_json(json: &Json) -> Self {
		match json {
			&Json::String(ref s) => match s.len() % 2 {
				0 => FromHex::from_hex(clean(s)).unwrap_or(vec![]),
				_ => FromHex::from_hex(&("0".to_string() + &(clean(s).to_string()))[..]).unwrap_or(vec![]),
			},
			_ => vec![],
		}
	}
}

impl FromJson for BTreeMap<H256, H256> {
	fn from_json(json: &Json) -> Self {
		match json {
			&Json::Object(ref o) => o.iter().map(|(key, value)| (x!(&u256_from_str(key)), x!(&U256::from_json(value)))).collect(),
			_ => BTreeMap::new(),
		}
	}
}

impl<T> FromJson for Vec<T> where T: FromJson {
	fn from_json(json: &Json) -> Self {
		match json {
			&Json::Array(ref o) => o.iter().map(|x|T::from_json(x)).collect(),
			_ => Vec::new(),
		}
	}
}

impl FromJson for u64 {
	fn from_json(json: &Json) -> Self {
		U256::from_json(json).low_u64()
	}
}

impl FromJson for u32 {
	fn from_json(json: &Json) -> Self {
		U256::from_json(json).low_u64() as u32
	}
}

impl FromJson for u16 {
	fn from_json(json: &Json) -> Self {
		U256::from_json(json).low_u64() as u16
	}
}

#[test]
fn u256_from_json() {
	let j = Json::from_str("{ \"dec\": \"10\", \"hex\": \"0x0a\", \"int\": 10 }").unwrap();

	let v: U256 = xjson!(&j["dec"]);
	assert_eq!(U256::from(10), v);
	let v: U256 = xjson!(&j["hex"]);
	assert_eq!(U256::from(10), v);
	let v: U256 = xjson!(&j["int"]);
	assert_eq!(U256::from(10), v);
}

#[test]
fn h256_from_json_() {
	let j = Json::from_str("{ \"with\": \"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\", \"without\": \"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\" }").unwrap();

	let v: H256 = xjson!(&j["with"]);
	assert_eq!(H256::from_str("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap(), v);
	let v: H256 = xjson!(&j["without"]);
	assert_eq!(H256::from_str("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap(), v);
}

#[test]
fn vec_u256_from_json() {
	let j = Json::from_str("{ \"array\": [ \"10\", \"0x0a\", 10] }").unwrap();

	let v: Vec<U256> = xjson!(&j["array"]);
	assert_eq!(vec![U256::from(10); 3], v);
}

#[test]
fn vec_h256_from_json_() {
	let j = Json::from_str("{ \"array\": [ \"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\", \"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\"] }").unwrap();

	let v: Vec<H256> = xjson!(&j["array"]);
	assert_eq!(vec![H256::from_str("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap(); 2], v);
}
