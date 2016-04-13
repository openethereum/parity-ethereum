// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use common::*;

/// Remove the `"0x"`, if present, from the left of `s`, returning the remaining slice.
pub fn clean(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" { &s[2..] } else { s }
}

fn u256_from_str(s: &str) -> U256 {
	if s.len() >= 2 && &s[0..2] == "0x" { U256::from_str(&s[2..]).unwrap_or_else(|_| U256::zero()) } else { U256::from_dec_str(s).unwrap_or_else(|_| U256::zero()) }
}

impl FromJson for Bytes {
	fn from_json(json: &Json) -> Self {
		match *json {
			Json::String(ref s) => match s.len() % 2 {
				0 => FromHex::from_hex(clean(s)).unwrap_or_else(|_| vec![]),
				_ => FromHex::from_hex(&("0".to_owned() + &(clean(s).to_owned()))[..]).unwrap_or_else(|_| vec![]),
			},
			_ => vec![],
		}
	}
}

impl FromJson for BTreeMap<H256, H256> {
	fn from_json(json: &Json) -> Self {
		match *json {
			Json::Object(ref o) => o.iter().map(|(key, value)| (x!(&u256_from_str(key)), x!(&U256::from_json(value)))).collect(),
			_ => BTreeMap::new(),
		}
	}
}

impl<T> FromJson for Vec<T>
    where T: FromJson,
{
	fn from_json(json: &Json) -> Self {
		match *json {
			Json::Array(ref o) => o.iter().map(|x| T::from_json(x)).collect(),
			_ => Vec::new(),
		}
	}
}

impl<T> FromJson for Option<T>
    where T: FromJson,
{
	fn from_json(json: &Json) -> Self {
		match *json {
			Json::String(ref o) if o.is_empty() => None,
			Json::Null => None,
			_ => Some(FromJson::from_json(json)),
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
fn h256_from_json() {
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
fn vec_h256_from_json() {
	let j = Json::from_str("{ \"array\": [ \"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\", \"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\"] }").unwrap();

	let v: Vec<H256> = xjson!(&j["array"]);
	assert_eq!(vec![H256::from_str("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap(); 2], v);
}

#[test]
fn simple_types() {
	let j = Json::from_str("{ \"null\": null, \"empty\": \"\", \"int\": 42, \"dec\": \"42\", \"hex\": \"0x2a\" }").unwrap();
	let v: u16 = xjson!(&j["int"]);
	assert_eq!(42u16, v);
	let v: u32 = xjson!(&j["dec"]);
	assert_eq!(42u32, v);
	let v: u64 = xjson!(&j["hex"]);
	assert_eq!(42u64, v);
}

#[test]
fn option_types() {
	let j = Json::from_str("{ \"null\": null, \"empty\": \"\", \"int\": 42, \"dec\": \"42\", \"hex\": \"0x2a\" }").unwrap();
	let v: Option<u16> = xjson!(&j["int"]);
	assert_eq!(Some(42u16), v);
	let v: Option<u16> = xjson!(&j["dec"]);
	assert_eq!(Some(42u16), v);
	let v: Option<u16> = xjson!(&j["null"]);
	assert_eq!(None, v);
	let v: Option<u16> = xjson!(&j["empty"]);
	assert_eq!(None, v);
}
