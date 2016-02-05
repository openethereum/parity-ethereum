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

//! json rlp tests
use rustc_serialize::*;
use super::{JsonTest, JsonLoader};
use util::*;

pub enum Operation {
	Append(Vec<u8>),
	AppendList(usize),
	AppendRaw(Vec<u8>, usize),
	AppendEmpty
}

impl Into<Operation> for json::Json {
	fn into(self) -> Operation {
		let obj = self.as_object().unwrap();
		match obj["operation"].as_string().unwrap().as_ref() {
			"append" => Operation::Append(hex_or_string(obj["value"].as_string().unwrap())),
			"append_list" => Operation::AppendList(obj["len"].as_u64().unwrap() as usize),
			"append_raw" => Operation::AppendRaw(hex_or_string(obj["value"].as_string().unwrap()), obj["len"].as_u64().unwrap() as usize),
			"append_empty" => Operation::AppendEmpty,
			other => { panic!("Unsupported opertation: {}", other); }
		}
	}
}

pub struct RlpStreamTest {
	loader: JsonLoader
}

impl JsonTest for RlpStreamTest {
	type Input = Vec<Operation>;
	type Output = Vec<u8>;

	fn new(data: &[u8]) -> Self {
		RlpStreamTest {
			loader: JsonLoader::new(data) 
		}
	}

	fn input(&self) -> Self::Input {
		self.loader.input().as_array().unwrap()
			.iter()
			.cloned()
			.map(|i| i.into())
			.collect()
	}

	fn output(&self) -> Self::Output {
		hex_or_string(self.loader.output().as_string().unwrap())
	}
}

