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

//! json trie tests
use std::collections::HashMap;
use rustc_serialize::*;
use super::{JsonTest, JsonLoader};
use util::*;

#[derive(RustcDecodable)]
struct RawOperation {
	operation: String,
	key: String,
	value: Option<String>
}

pub enum Operation {
	Insert(Vec<u8>, Vec<u8>),
	Remove(Vec<u8>)
}

impl Into<Operation> for RawOperation {
	fn into(self) -> Operation {
		match self.operation.as_ref() {
			"insert" => Operation::Insert(hex_or_string(&self.key), hex_or_string(&self.value.unwrap())),
			"remove" => Operation::Remove(hex_or_string(&self.key)),
			other => panic!("invalid operation type: {}", other)
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

