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

use rustc_hex::FromHex as RustcFromHex;
use bloomchain::Bloom;

pub trait FromHex {
	fn from_hex(s: &str) -> Self where Self: Sized;
}

impl FromHex for Bloom {
	fn from_hex(s: &str) -> Self {
		let mut res = [0u8; 256];
		let v = s.from_hex().unwrap();
		assert_eq!(res.len(), v.len());
		res.copy_from_slice(&v);
		From::from(res)
	}
}
