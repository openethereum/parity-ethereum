// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Wrapper for view rlp expected to be valid with debug info

use rlp::{Rlp, Decodable, DecoderError};

/// Wrapper for trusted rlp, which is expected to be valid, for use in views
/// When created with view!, records the file and line where it was created for debugging
pub struct ViewRlp<'a> {
	/// Wrapped Rlp, expected to be valid
	pub rlp: Rlp<'a>,
	file: &'a str,
	line: u32,
}

impl<'a, 'view> ViewRlp<'a> where 'a : 'view {
	#[doc(hidden)]
	pub fn new(bytes: &'a [u8], file: &'a str, line: u32) -> Self {
		ViewRlp {
			rlp: Rlp::new(bytes),
			file,
			line
		}
	}

	/// Returns a new instance replacing existing rlp with new rlp, maintaining debug info
	fn new_from_rlp(&self, rlp: Rlp<'a>) -> Self {
		ViewRlp {
			rlp,
			file: self.file,
			line: self.line
		}
	}

	fn maybe_at(&self, index: usize) -> Option<ViewRlp<'a>> {
		self.rlp.at(index)
			.map(|rlp| self.new_from_rlp(rlp))
			.ok()
	}

	fn expect_valid_rlp<T>(&self, r: Result<T, DecoderError>) -> T {
		r.unwrap_or_else(|e| panic!(
			"View rlp is trusted and should be valid. Constructed in {} on line {}: {}",
			self.file,
			self.line,
			e
		))
	}

	/// Returns rlp at the given index, panics if no rlp at that index
	pub fn at(&self, index: usize) -> ViewRlp<'a> {
		let rlp = self.expect_valid_rlp(self.rlp.at(index));
		self.new_from_rlp(rlp)
	}

	/// Returns an iterator over all rlp values
	pub fn iter(&'view self) -> ViewRlpIterator<'a, 'view> {
		self.into_iter()
	}

	/// Returns decoded value of this rlp, panics if rlp not valid
	pub fn as_val<T>(&self) -> T where T: Decodable {
		self.expect_valid_rlp(self.rlp.as_val())
	}

	/// Returns decoded value at the given index, panics not present or valid at that index
	pub fn val_at<T>(&self, index: usize) -> T where T : Decodable {
		self.expect_valid_rlp(self.rlp.val_at(index))
	}

	/// Returns decoded list of values, panics if rlp is invalid
	pub fn list_at<T>(&self, index: usize) -> Vec<T> where T: Decodable {
		self.expect_valid_rlp(self.rlp.list_at(index))
	}

	/// Returns the number of items in the rlp, panics if it is not a list of rlp values
	pub fn item_count(&self) -> usize {
		self.expect_valid_rlp(self.rlp.item_count())
	}

	/// Returns raw rlp bytes
	pub fn as_raw(&'view self) -> &'a [u8] {
		self.rlp.as_raw()
	}
}

/// Iterator over rlp-slice list elements.
pub struct ViewRlpIterator<'a, 'view> where 'a: 'view {
	rlp: &'view ViewRlp<'a>,
	index: usize,
}

impl<'a, 'view> IntoIterator for &'view ViewRlp<'a> where 'a: 'view {
	type Item = ViewRlp<'a>;
	type IntoIter = ViewRlpIterator<'a, 'view>;

	fn into_iter(self) -> Self::IntoIter {
		ViewRlpIterator {
			rlp: self,
			index: 0,
		}
	}
}

impl<'a, 'view> Iterator for ViewRlpIterator<'a, 'view> {
	type Item = ViewRlp<'a>;

	fn next(&mut self) -> Option<ViewRlp<'a>> {
		let index = self.index;
		let result = self.rlp.maybe_at(index);
		self.index += 1;
		result
	}
}

#[macro_export]
/// Create a view into RLP-data
macro_rules! view {
	($view: ident, $bytes: expr) => {
		$view::new($crate::views::ViewRlp::new($bytes, file!(), line!()))
	};
}
