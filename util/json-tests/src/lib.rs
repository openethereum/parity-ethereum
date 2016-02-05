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

extern crate rustc_serialize;
extern crate glob;

use std::str::from_utf8;
use std::path::*;
use std::io::prelude::*;
use std::fs::File;
use glob::glob;
use rustc_serialize::*;

mod util;
pub mod trie;
pub mod rlp;

pub trait JsonTest: Sized {
	type Input;
	type Output;

	fn new(data: &[u8]) -> Self;
	fn input(&self) -> Self::Input;
	fn output(&self) -> Self::Output;
}

pub struct JsonLoader {
	json: json::Json
}

impl JsonTest for JsonLoader {
	type Input = json::Json;
	type Output = json::Json;

	fn new(data: &[u8]) -> Self {
		JsonLoader {
			json: json::Json::from_str(from_utf8(data).unwrap()).unwrap()
		}
	}
	fn input(&self) -> Self::Input {
		self.json.as_object().unwrap()["input"].clone()
	}

	fn output(&self) -> Self::Output {
		self.json.as_object().unwrap()["output"].clone()
	}
}

pub fn execute_test<T, F>(data: &[u8], f: &mut F) where T: JsonTest, F: FnMut(T::Input, T::Output) {
	let test = T::new(data);
	f(test.input(), test.output())
}

pub fn execute_test_from_file<T, F>(path: &Path, f: &mut F) where T: JsonTest, F: FnMut(T::Input, T::Output) {
	let mut file = File::open(path).unwrap();
	let mut buffer = vec![];
	let _  = file.read_to_end(&mut buffer);
	let test = T::new(&buffer);
	f(test.input(), test.output())
}

pub fn execute_tests_from_directory<T, F>(pattern: &str, f: &mut F) where T: JsonTest, F: FnMut(String, T::Input, T::Output) {
	for path in glob(pattern).unwrap().filter_map(Result::ok) {
		execute_test_from_file::<T, _>(&path, &mut | input, output | {
			f(path.to_str().unwrap().to_string(), input, output);
		});
	}
}

