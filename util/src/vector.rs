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

//! Vector extensions.

/// Returns len of prefix shared with elem
///
/// ```rust
///	extern crate ethcore_util as util;
///	use util::vector::SharedPrefix;
///
///	fn main () {
///		let a = vec![1,2,3,3,5];
///		let b = vec![1,2,3];
///		assert_eq!(a.shared_prefix_len(&b), 3);
///	}
/// ```
pub trait SharedPrefix<T> {
	/// Get common prefix length
	fn shared_prefix_len(&self, elem: &[T]) -> usize;
}

impl<T> SharedPrefix<T> for [T] where T: Eq {
	fn shared_prefix_len(&self, elem: &[T]) -> usize {
		use std::cmp;
		let len = cmp::min(self.len(), elem.len());
		(0..len).take_while(|&i| self[i] == elem[i]).count()
	}
}

#[cfg(test)]
mod test {
	use vector::SharedPrefix;

	#[test]
	fn test_shared_prefix() {
		let a = vec![1,2,3,4,5,6];
		let b = vec![4,2,3,4,5,6];
		assert_eq!(a.shared_prefix_len(&b), 0);
	}

	#[test]
	fn test_shared_prefix2() {
		let a = vec![1,2,3,3,5];
		let b = vec![1,2,3];
		assert_eq!(a.shared_prefix_len(&b), 3);
	}

	#[test]
	fn test_shared_prefix3() {
		let a = vec![1,2,3,4,5,6];
		let b = vec![1,2,3,4,5,6];
		assert_eq!(a.shared_prefix_len(&b), 6);
	}
}
