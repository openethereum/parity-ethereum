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

use std;

#[allow(unused)]
pub enum Error {
	ProcessError(std::io::Error),
	FfiNull(std::ffi::NulError),
	WindowsShellExecute,
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::ProcessError(err)
	}
}

impl From<std::ffi::NulError> for Error {
	fn from(err: std::ffi::NulError) -> Self {
		Error::FfiNull(err)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		match *self {
			Error::ProcessError(ref e) => write!(f, "{}", e),
			Error::FfiNull(ref e) => write!(f, "{}", e),
			Error::WindowsShellExecute => write!(f, "WindowsShellExecute failed"),
		}
	}
}

#[cfg(windows)]
pub fn open(url: &str) -> Result<(), Error> {
	use std::ffi::CString;
	use std::ptr;
	use winapi::um::shellapi::ShellExecuteA;
	use winapi::um::winuser::SW_SHOWNORMAL as Normal;
	use winapi::shared::minwindef::INT;

	const WINDOWS_SHELL_EXECUTE_SUCCESS: i32 = 32;

	let h_instance = unsafe {
		ShellExecuteA(ptr::null_mut(),
			CString::new("open")?.as_ptr(),
			CString::new(url.to_owned().replace("\n", "%0A"))?.as_ptr(),
			ptr::null(),
			ptr::null(),
			Normal) as INT
};
	// https://msdn.microsoft.com/en-us/library/windows/desktop/bb762153(v=vs.85).aspx
	// `ShellExecute` returns a value greater than 32 on success
	if h_instance > WINDOWS_SHELL_EXECUTE_SUCCESS { Ok(()) } else { Err(Error::WindowsShellExecute) }
}

#[cfg(any(target_os="macos", target_os="freebsd"))]
pub fn open(url: &str) -> Result<(), Error> {
	let _ = std::process::Command::new("open").arg(url).spawn()?;
	Ok(())
}

#[cfg(target_os="linux")]
pub fn open(url: &str) -> Result<(), Error> {
	let _ = std::process::Command::new("xdg-open").arg(url).spawn()?;
	Ok(())
}

#[cfg(target_os="android")]
pub fn open(_url: &str) {
	// TODO: While it is generally always bad to leave a function implemented, there is not much
	//		 more we can do here. This function will eventually be removed when we compile Parity
	//		 as a library and not as a full binary.
}
