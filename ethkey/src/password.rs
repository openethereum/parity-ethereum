// Copyright 2018 Parity Technologies (UK) Ltd.
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

use std::ptr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password(String);

impl Password {
	pub fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}

	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl Drop for Password {
	fn drop(&mut self) {
		let vec = unsafe {
			self.0.as_mut_vec()
		};
		let n = vec.len();
		let p = vec.as_mut_ptr();
		for i in 0..n {
			unsafe {
				ptr::write_volatile(p.offset(i as isize), 0)
			}
		}
	}
}

impl From<String> for Password {
	fn from(s: String) -> Password {
		Password(s)
	}
}

impl<'a> From<&'a str> for Password {
	fn from(s: &'a str) -> Password {
		Password::from(String::from(s))
	}
}

