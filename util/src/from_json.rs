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

//! Coversion from json.

use standard::*;
use bigint::uint::*;

#[macro_export]
macro_rules! xjson {
	( $x:expr ) => {
		FromJson::from_json($x)
	}
}

/// Trait allowing conversion from a JSON value.
pub trait FromJson {
	/// Convert a JSON value to an instance of this type.
	fn from_json(json: &Json) -> Self;
}

impl FromJson for U256 {
	fn from_json(json: &Json) -> Self {
		match *json {
			Json::String(ref s) => {
				if s.len() >= 2 && &s[0..2] == "0x" {
					FromStr::from_str(&s[2..]).unwrap_or_else(|_| Default::default())
				} else {
					Uint::from_dec_str(s).unwrap_or_else(|_| Default::default())
				}
			},
			Json::U64(u) => From::from(u),
			Json::I64(i) => From::from(i as u64),
			_ => Uint::zero(),
		}
	}
}
