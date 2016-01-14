use common::*;

pub fn clean(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" {
		&s[2..]
	} else {
		s
	}
}

pub fn u256_from_str(s: &str) -> U256 {
	if s.len() >= 2 && &s[0..2] == "0x" {
		U256::from_str(&s[2..]).unwrap_or(U256::from(0))
	} else {
		U256::from_dec_str(s).unwrap_or(U256::from(0))
	}
}

pub fn bytes_from_json(json: &Json) -> Bytes {
	let s = json.as_string().unwrap_or("");
	if s.len() % 2 == 1 {
		FromHex::from_hex(&("0".to_string() + &(clean(s).to_string()))[..]).unwrap_or(vec![])
	} else {
		FromHex::from_hex(clean(s)).unwrap_or(vec![])
	}
}

pub fn address_from_json(json: &Json) -> Address {
	From::from(json.as_string().unwrap_or("0000000000000000000000000000000000000000"))
}

pub fn h256_from_json(json: &Json) -> H256 {
	let s = json.as_string().unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
	if s.len() % 2 == 1 {
		h256_from_hex(&("0".to_string() + &(clean(s).to_string()))[..])
	} else {
		h256_from_hex(clean(s))
	}
}

pub fn vec_h256_from_json(json: &Json) -> Vec<H256> {
	json.as_array().unwrap().iter().map(&h256_from_json).collect()
}

pub fn map_h256_h256_from_json(json: &Json) -> BTreeMap<H256, H256> {
	json.as_object().unwrap().iter().fold(BTreeMap::new(), |mut m, (key, value)| {
		m.insert(H256::from(&u256_from_str(key)), H256::from(&U256::from_json(value)));
		m
	})
}

pub fn usize_from_json(json: &Json) -> usize {
	U256::from_json(json).low_u64() as usize
}

pub fn u64_from_json(json: &Json) -> u64 {
	U256::from_json(json).low_u64()
}

pub fn u32_from_json(json: &Json) -> u32 {
	U256::from_json(json).low_u32()
}

pub fn u16_from_json(json: &Json) -> u16 {
	U256::from_json(json).low_u32() as u16
}

pub fn u8_from_json(json: &Json) -> u8 {
	U256::from_json(json).low_u32() as u8
}

impl<T> FromJson for Vec<T> where T: FromJson {
	fn from_json(json: &Json) -> Self {
		json.as_array().unwrap().iter().map(|x|T::from_json(x)).collect()
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

impl FromJson for u8 {
	fn from_json(json: &Json) -> Self {
		U256::from_json(json).low_u64() as u8
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
