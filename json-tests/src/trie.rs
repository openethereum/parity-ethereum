//! json trie tests
use std::collections::HashMap;
use rustc_serialize::*;
use rustc_serialize::hex::FromHex;
use super::{JsonTest, JsonLoader};

enum OperationType {
	Insert,
	Remove
}

impl Decodable for OperationType {
	fn decode<D>(d: &mut D) -> Result<OperationType, D::Error> where D: Decoder {
		match try!(String::decode(d)).as_ref() {
			"insert" => Ok(OperationType::Insert),
			"remove" => Ok(OperationType::Remove),
			other => panic!("invalid operation type: {}", other)
		}
	}
}

#[derive(RustcDecodable)]
struct RawOperation {
	operation: OperationType,
	key: String,
	value: Option<String>
}

pub enum Operation {
	Insert(Vec<u8>, Vec<u8>),
	Remove(Vec<u8>)
}

fn hex_or_string(s: &str) -> Vec<u8> {
	match s.starts_with("0x") {
		true => s[2..].from_hex().unwrap(),
		false => From::from(s)
	}
}

impl Into<Operation> for RawOperation {
	fn into(self) -> Operation {
		match self.operation {
			OperationType::Insert => Operation::Insert(hex_or_string(&self.key), hex_or_string(&self.value.unwrap())),
			OperationType::Remove => Operation::Remove(hex_or_string(&self.key))
		}
	}
}

pub struct TrieTest {
	loader: JsonLoader
}

impl JsonTest for TrieTest {
	type Input = Vec<Operation>;
	type Output = Vec<u8>;
	
	fn new(data: &[u8]) -> Self {
		TrieTest {
			loader: JsonLoader::new(data) 
		}
	}

	fn input(&self) -> Self::Input {
		let mut decoder = json::Decoder::new(self.loader.input());
		let raw: Vec<RawOperation> = Decodable::decode(&mut decoder).unwrap();
		raw.into_iter()
			.map(|i| i.into())
			.collect()
	}

	fn output(&self) -> Self::Output {
		hex_or_string(self.loader.output().as_string().unwrap())
	}
}

pub struct TriehashTest {
	trietest: TrieTest
}

impl JsonTest for TriehashTest {
	type Input = Vec<(Vec<u8>, Vec<u8>)>;
	type Output = Vec<u8>;

	fn new(data: &[u8]) -> Self {
		TriehashTest {
			trietest: TrieTest::new(data)
		}
	}

	fn input(&self) -> Self::Input {
		self.trietest.input()
			.into_iter()
			.fold(HashMap::new(), | mut map, o | {
				match o {
					Operation::Insert(k, v) => map.insert(k, v),
					Operation::Remove(k) => map.remove(&k)
				};
				map
			})
			.into_iter()
			.map(|p| { p })
			.collect()
	}

	fn output(&self) -> Self::Output {
		self.trietest.output()
	}
}

