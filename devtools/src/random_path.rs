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

//! Random path

use std::path::*;
use std::fs;
use std::env;
use rand::random;

pub struct RandomTempPath {
	path: PathBuf
}

pub fn random_filename() -> String {
	(0..8).map(|_| ((random::<f32>() * 26.0) as u8 + 97) as char).collect()
}

impl RandomTempPath {
	pub fn new() -> RandomTempPath {
		let mut dir = env::temp_dir();
		dir.push(random_filename());
		RandomTempPath {
			path: dir.clone()
		}
	}

	pub fn create_dir() -> RandomTempPath {
		let mut dir = env::temp_dir();
		dir.push(random_filename());
		fs::create_dir_all(dir.as_path()).unwrap();
		RandomTempPath {
			path: dir.clone()
		}
	}

	pub fn as_path(&self) -> &PathBuf {
		&self.path
	}

	pub fn as_str(&self) -> &str {
		self.path.to_str().unwrap()
	}
}

impl Drop for RandomTempPath {
	fn drop(&mut self) {
		if let Err(e) = fs::remove_dir_all(self.as_path()) {
			panic!("failed to remove temp directory, probably something failed to destroyed ({})", e);
		}
	}
}

#[test]
fn creates_dir() {
	let temp = RandomTempPath::create_dir();
	assert!(fs::metadata(temp.as_path()).unwrap().is_dir());
}

#[test]
fn destroys_dir() {
	let path_buf = {
		let temp = RandomTempPath::create_dir();
		assert!(fs::metadata(temp.as_path()).unwrap().is_dir());
		let path_buf = temp.as_path().to_path_buf();
		path_buf
	};

	assert!(fs::metadata(&path_buf).is_err());
}

#[test]
fn provides_random() {
	let temp = RandomTempPath::create_dir();
	assert!(temp.as_path().to_str().is_some());
}
