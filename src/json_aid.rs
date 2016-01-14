use common::*;

pub fn clean(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" {
		&s[2..]
	} else {
		s
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

pub fn u256_from_str(s: &str) -> U256 {
	if s.len() >= 2 && &s[0..2] == "0x" {
		U256::from_str(&s[2..]).unwrap_or(U256::from(0))
	} else {
		U256::from_dec_str(s).unwrap_or(U256::from(0))
	}
}

pub fn u256_from_json(json: &Json) -> U256 {
	u256_from_str(json.as_string().unwrap_or(""))
}

pub fn usize_from_json(json: &Json) -> usize {
	u256_from_json(json).low_u64() as usize
}

pub fn u64_from_json(json: &Json) -> u64 {
	u256_from_json(json).low_u64()
}

pub fn u32_from_json(json: &Json) -> u32 {
	u256_from_json(json).low_u32()
}

pub fn u16_from_json(json: &Json) -> u16 {
	u256_from_json(json).low_u32() as u16
}

pub fn u8_from_json(json: &Json) -> u8 {
	u256_from_json(json).low_u32() as u8
}
