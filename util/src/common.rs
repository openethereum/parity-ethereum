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

//! Utils common types and macros global reexport.

pub use standard::*;
pub use from_json::*;
pub use error::*;
pub use bytes::*;
pub use vector::*;
pub use numbers::*;
pub use sha3::*;

#[macro_export]
macro_rules! hash_map {
	( $( $x:expr => $y:expr ),* ) => {
		vec![ $( ($x, $y) ),* ].into_iter().collect::<HashMap<_, _>>()
	}
}

#[macro_export]
macro_rules! hash_mapx {
	( $( $x:expr => $y:expr ),* ) => {
		vec![ $( ( From::from($x), From::from($y) ) ),* ].into_iter().collect::<HashMap<_, _>>()
	}
}

#[macro_export]
macro_rules! map {
	( $( $x:expr => $y:expr ),* ) => {
		vec![ $( ($x, $y) ),* ].into_iter().collect::<BTreeMap<_, _>>()
	}
}

#[macro_export]
macro_rules! mapx {
	( $( $x:expr => $y:expr ),* ) => {
		vec![ $( ( From::from($x), From::from($y) ) ),* ].into_iter().collect::<BTreeMap<_, _>>()
	}
}

#[macro_export]
macro_rules! x {
	( $x:expr ) => {
		From::from($x)
	}
}

#[macro_export]
macro_rules! xx {
	( $x:expr ) => {
		From::from(From::from($x))
	}
}

#[macro_export]
macro_rules! flush {
	($($arg:tt)*) => ($crate::flush(format!("{}", format_args!($($arg)*))));
}

#[macro_export]
macro_rules! flushln {
	($fmt:expr) => (flush!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (flush!(concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn flush(s: String) {
	::std::io::stdout().write(s.as_bytes()).unwrap();
	::std::io::stdout().flush().unwrap();
}

#[test]
fn test_flush() {
	flushln!("hello_world {:?}", 1);
}
