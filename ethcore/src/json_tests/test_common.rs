// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

pub use util::*;

macro_rules! test {
	($name: expr) => {
		assert!(do_json_test(include_bytes!(concat!("../../res/ethereum/tests/", $name, ".json"))).is_empty());
	}
}

#[macro_export]
macro_rules! declare_test {
	(ignore => $id: ident, $name: expr) => {
		#[ignore]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	};
	(heavy => $id: ident, $name: expr) => {
		#[cfg(feature = "test-heavy")]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	};
	($id: ident, $name: expr) => {
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	}
}
