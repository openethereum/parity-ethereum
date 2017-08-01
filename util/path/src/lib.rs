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

//! Path utilities
use std::path::Path;

/// Restricts the permissions of given path only to the owner.
#[cfg(unix)]
pub fn restrict_permissions_owner(file_path: &Path, write: bool, executable: bool) -> Result<(), String>  {
	let perms = ::std::os::unix::fs::PermissionsExt::from_mode(0o400 + write as u32 * 0o200 + executable as u32 * 0o100);
	::std::fs::set_permissions(file_path, perms).map_err(|e| format!("{:?}", e))
}

/// Restricts the permissions of given path only to the owner.
#[cfg(not(unix))]
pub fn restrict_permissions_owner(_file_path: &Path, _write: bool, _executable: bool) -> Result<(), String>  {
	//TODO: implement me
	Ok(())
}

