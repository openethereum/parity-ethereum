// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! RPC unit test moduleS

pub mod helpers;

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
	(iter $file:expr) => {{
		const RAW_DATA: &'static [u8] =
			include_bytes!(concat!("../../../../ethcore/res/ethereum/tests/", $file, ".json"));
		::ethjson::blockchain::Test::load(RAW_DATA).unwrap().into_iter()
	}};

	($file:expr) => {{
		extract_chain!(iter $file).filter(|&(_, ref t)| t.network == ForkSpec::Frontier).next().unwrap().1
	}};
}

macro_rules! register_test {
	($name:ident, $cb:expr, $file:expr) => {
		#[test]
		fn $name() {
			for (name, chain) in extract_chain!(iter $file).filter(|&(_, ref t)| t.network == ForkSpec::Frontier) {
				$cb(name, chain);
			}
		}
	};
}

#[cfg(test)]
mod mocked;
#[cfg(test)]
mod eth;
