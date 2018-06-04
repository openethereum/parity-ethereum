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

//! Directory helper functions
use std::env;

/// Replaces `$HOME` str with home directory path.
pub fn replace_home(base: &str, arg: &str) -> String {
	// the $HOME directory on mac os should be `~/Library` or `~/Library/Application Support`
	let r = arg.replace("$HOME", env::home_dir().unwrap().to_str().unwrap());
	let r = r.replace("$BASE", base);
	r.replace("/", &::std::path::MAIN_SEPARATOR.to_string())
}

/// Replaces `$HOME` str with home directory path and `$LOCAL` with local path.
pub fn replace_home_and_local(base: &str, local: &str, arg: &str) -> String {
	let r = replace_home(base, arg);
	r.replace("$LOCAL", local)
}
