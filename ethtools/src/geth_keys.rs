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

//! Geth keys import/export tool

use util::hash::*;
use std::path::Path;
use std::result::*;
use std::fs;
use std::str::FromStr;

/// Enumerates all geth keys in the directory and returns collection of tuples `(accountId, filename)`
pub fn enumerate_geth_keys(path: &Path) -> Result<Vec<(Address, String)>, ::std::io::Error> {
	let mut entries = Vec::new();
	for entry in try!(fs::read_dir(path)) {
		let entry = try!(entry);
		if !try!(fs::metadata(entry.path())).is_dir() {
			match entry.file_name().to_str() {
				Some(name) => {
					let parts: Vec<&str> = name.split("--").collect();
					if parts.len() != 3 { continue; }
					match Address::from_str(parts[2]) {
						Ok(account_id) => { entries.push((account_id, name.to_owned())); }
						Err(e) => { panic!("error: {:?}", e); }
					}
				},
				None => { continue; }
			};
		}
	}
	Ok(entries)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;

	#[test]
	fn can_enumerate() {
		let keys = enumerate_geth_keys(Path::new("res/geth_keystore")).unwrap();
		assert_eq!(2, keys.len());
	}
}
