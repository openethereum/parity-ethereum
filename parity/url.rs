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

//! Cross-platform open url in default browser

use std::{io, fmt, process};

#[derive(Debug)]
pub enum Error {
	ProcessError(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::ProcessError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::ProcessError(ref e) => write!(f, "{}", e),
        }
    }
}

#[cfg(windows)]
pub fn open(url: &str) -> Result<(), Error> {
	use std::ffi::CString;
	use std::ptr;
	use winapi::um::shellapi::ShellExecuteA;
	use winapi::um::winuser::SW_SHOWNORMAL as Normal;

	unsafe {
		ShellExecuteA(ptr::null_mut(),
			CString::new("open").unwrap().as_ptr(),
			CString::new(url.to_owned().replace("\n", "%0A")).unwrap().as_ptr(),
			ptr::null(),
			ptr::null(),
			Normal);
	}
	Ok(())
}

#[cfg(any(target_os="macos", target_os="freebsd"))]
pub fn open(url: &str) -> Result<(), Error> {
	let _ = process::Command::new("open").arg(url).spawn()?;
	Ok(())
}

#[cfg(target_os="linux")]
pub fn open(url: &str) -> Result<(), Error> {
	let _ = process::Command::new("xdg-open").arg(url).spawn()?;
	Ok(())
}

#[cfg(target_os="android")]
pub fn open(_url: &str) {
	// TODO: While it is generally always bad to leave a function implemented, there is not much
	//		 more we can do here. This function will eventually be removed when we compile Parity
	//		 as a library and not as a full binary.
}
