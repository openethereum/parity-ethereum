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

use std::path::Path;
use util::journaldb::Algorithm;

pub struct UserDefaults {
	pub pruning: Algorithm,
	pub tracing: bool,
}

impl Default for UserDefaults {
	fn default() -> Self {
		UserDefaults {
			pruning: Algorithm::default(),
			tracing: false,
		}
	}
}

impl UserDefaults {
	pub fn load<P>(_path: P) -> Result<Self, String> where P: AsRef<Path> {
		unimplemented!();
	}

	pub fn save<P>(self, _path: P) -> Result<(), String> where P: AsRef<Path> {
		unimplemented!();
	}
}
