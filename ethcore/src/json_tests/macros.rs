// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Helper macros for running the `JSON tests`


/// Similar to `print!` but flushes stdout in order to ensure the output is emitted immediately.
#[macro_export]
macro_rules! flushed_write {
	($arg:expr) => ($crate::json_tests::macros::write_and_flush($arg.into()));
	($($arg:tt)*) => ($crate::json_tests::macros::write_and_flush(format!("{}", format_args!($($arg)*))));
}

/// Similar to `println!` but flushes stdout in order to ensure the output is emitted immediately.
#[macro_export]
macro_rules! flushed_writeln {
	($fmt:expr) => (flushed_write!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (flushed_write!(concat!($fmt, "\n"), $($arg)*));
}

/// Write to stdout and flush (ignores errors)
#[doc(hidden)]
pub fn write_and_flush(s: String) {
	if let Err(err) = std::io::Write::write_all(&mut std::io::stdout(), s.as_bytes()) {
		error!(target: "json_tests", "io::Write::write_all to stdout failed because of: {:?}", err);
	}
	if let Err(err) = std::io::Write::flush(&mut std::io::stdout()) {
		error!(target: "json_tests", "io::Write::flush stdout failed because of: {:?}", err);
	}
}
