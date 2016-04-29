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

use std;
use ethcore;
use util::UtilError;
use std::process::exit;

#[macro_export]
macro_rules! die {
	($($arg:tt)*) => (die_with_message(&format!("{}", format_args!($($arg)*))));
}

pub fn die_with_error(module: &'static str, e: ethcore::error::Error) -> ! {
	use ethcore::error::Error;

	match e {
		Error::Util(UtilError::StdIo(e)) => die_with_io_error(module, e),
		_ => die!("{}: {:?}", module, e),
	}
}

pub fn die_with_io_error(module: &'static str, e: std::io::Error) -> ! {
	match e.kind() {
		std::io::ErrorKind::PermissionDenied => {
			die!("{}: No permissions to bind to specified port.", module)
		},
		std::io::ErrorKind::AddrInUse => {
			die!("{}: Specified address is already in use. Please make sure that nothing is listening on the same port or try using a different one.", module)
		},
		std::io::ErrorKind::AddrNotAvailable => {
			die!("{}: Could not use specified interface or given address is invalid.", module)
		},
		_ => die!("{}: {:?}", module, e),
	}
}

pub fn die_with_message(msg: &str) -> ! {
	println!("ERROR: {}", msg);
	exit(1);
}
