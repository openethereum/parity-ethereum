// Copyright 2016 Ethcore (UK) Ltd.
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

//! Integration tests for the JSONRPC APIs

// extract a chain from the given JSON file,
// stored in ethcore/res/ethereum/tests/.
//
// usage:
//     `extract_chain!("Folder/File")` will load Folder/File.json and extract
//     the first block chain stored within.
//
//     `extract_chain!("Folder/File", "with_name")` will load Folder/File.json and
//     extract the chain with that name. This will panic if no chain by that name
//     is found.
macro_rules! extract_chain {
	($file:expr, $name:expr) => {{
		const RAW_DATA: &'static [u8] =
			include_bytes!(concat!("../../../../../ethcore/res/ethereum/tests/", $file, ".json"));
		let mut chain = None;
		for (name, c) in ::ethjson::blockchain::Test::load(RAW_DATA).unwrap() {
			if name == $name {
				chain = Some(c);
				break;
			}
		}
		chain.unwrap()
	}};

	($file:expr) => {{
		const RAW_DATA: &'static [u8] =
			include_bytes!(concat!("../../../../../ethcore/res/ethereum/tests/", $file, ".json"));

		::ethjson::blockchain::Test::load(RAW_DATA)
			.unwrap().into_iter().next().unwrap().1
	}};
}

mod eth;