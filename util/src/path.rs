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

//! Path utilities

/// Default ethereum paths
pub mod ethereum {
	use std::path::PathBuf;

	#[cfg(target_os = "macos")]
	/// Default path for ethereum installation on Mac Os
	pub fn default() -> PathBuf {
		let mut home = ::std::env::home_dir().expect("Failed to get home dir");
		home.push("Library");
		home.push("Ethereum");
		home
	}

	#[cfg(windows)]
	/// Default path for ethereum installation on Windows
	pub fn default() -> PathBuf {
		let mut home = ::std::env::home_dir().expect("Failed to get home dir");
		home.push("AppData");
		home.push("Roaming");
		home.push("Ethereum");
		home
	}

	#[cfg(not(any(target_os = "macos", windows)))]
	/// Default path for ethereum installation on posix system which is not Mac OS
	pub fn default() -> PathBuf {
		let mut home = ::std::env::home_dir().expect("Failed to get home dir");
		home.push(".ethereum");
		home
	}

	/// Get the specific folder inside default ethereum installation
	pub fn with_default(s: &str) -> PathBuf {
		let mut pth = default();
		pth.push(s);
		pth
	}
}
