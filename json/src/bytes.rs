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

//! Lenient bytes json deserialization for test json files.

use rustc_serialize::hex::FromHex;
use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;

/// Lenient bytes json deserialization for test json files.
#[derive(Default, Debug, PartialEq)]
pub struct Bytes(Vec<u8>);

impl Into<Vec<u8>> for Bytes {
	fn into(self) -> Vec<u8> {
		self.0
	}
}

impl Deserialize for Bytes {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
		where D: Deserializer {
		deserializer.deserialize(BytesVisitor)
	}
}

struct BytesVisitor;

impl Visitor for BytesVisitor {
	type Value = Bytes;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		let v = match value.len() {
			0 => vec![],
			2 if value.starts_with("0x") => vec![],
			_ if value.starts_with("0x") => try!(FromHex::from_hex(&value[2..]).map_err(|_| Error::custom("Invalid hex value."))),
			_ => try!(FromHex::from_hex(value).map_err(|_| Error::custom("Invalid hex value")))
		};
		Ok(Bytes(v))
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

#[cfg(test)]
//mod test {
	//use std::str::FromStr;
	//use serde_json;
	//use util::hash::H256;
	//use hash::Hash;

	//#[test]
	//fn uint_deserialization() {
		//let s = r#"["", "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae"]"#;
		//let deserialized: Vec<Hash> = serde_json::from_str(s).unwrap();
		//assert_eq!(deserialized, vec![
				   //Hash(H256::from(0)),
				   //Hash(H256::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae").unwrap())
		//]);
	//}

	//#[test]
	//fn uint_into() {
		//assert_eq!(H256::from(0), Hash(H256::from(0)).into());
	//}
//}
