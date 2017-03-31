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

use std::fmt;
use {UntrustedRlp, PayloadInfo, Prototype, Decodable};

impl<'a> From<UntrustedRlp<'a>> for Rlp<'a> {
	fn from(rlp: UntrustedRlp<'a>) -> Rlp<'a> {
		Rlp { rlp: rlp }
	}
}

/// Data-oriented view onto trusted rlp-slice.
///
/// Unlikely to `UntrustedRlp` doesn't bother you with error
/// handling. It assumes that you know what you are doing.
#[derive(Debug)]
pub struct Rlp<'a> {
	rlp: UntrustedRlp<'a>
}

impl<'a> fmt::Display for Rlp<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", self.rlp)
	}
}

impl<'a, 'view> Rlp<'a> where 'a: 'view {
	/// Create a new instance of `Rlp`
	pub fn new(bytes: &'a [u8]) -> Rlp<'a> {
		Rlp {
			rlp: UntrustedRlp::new(bytes)
		}
	}

	/// The raw data of the RLP as slice.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let dog = rlp.at(1).as_raw();
	/// 	assert_eq!(dog, &[0x83, b'd', b'o', b'g']);
	/// }
	/// ```
	pub fn as_raw(&'view self) -> &'a [u8] {
		self.rlp.as_raw()
	}

	/// Get the prototype of the RLP.
	pub fn prototype(&self) -> Prototype {
		self.rlp.prototype().unwrap()
	}

	/// Get payload info.
	pub fn payload_info(&self) -> PayloadInfo {
		self.rlp.payload_info().unwrap()
	}

	/// Get underlieing data.
	pub fn data(&'view self) -> &'a [u8] {
		self.rlp.data().unwrap()
	}

	/// Returns number of RLP items.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert_eq!(rlp.item_count(), 2);
	/// 	let view = rlp.at(1);
	/// 	assert_eq!(view.item_count(), 0);
	/// }
	/// ```
	pub fn item_count(&self) -> usize {
		self.rlp.item_count().unwrap_or(0)
	}

	/// Returns the number of bytes in the data, or zero if it isn't data.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert_eq!(rlp.size(), 0);
	/// 	let view = rlp.at(1);
	/// 	assert_eq!(view.size(), 3);
	/// }
	/// ```
	pub fn size(&self) -> usize {
		self.rlp.size()
	}

	/// Get view onto RLP-slice at index.
	///
	/// Caches offset to given index, so access to successive
	/// slices is faster.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let dog: String = rlp.at(1).as_val();
	/// 	assert_eq!(dog, "dog".to_string());
	/// }
	/// ```
	pub fn at(&'view self, index: usize) -> Rlp<'a> {
		From::from(self.rlp.at(index).unwrap())
	}

	/// No value
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.is_null());
	/// }
	/// ```
	pub fn is_null(&self) -> bool {
		self.rlp.is_null()
	}

	/// Contains a zero-length string or zero-length list.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc0];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.is_empty());
	/// }
	/// ```
	pub fn is_empty(&self) -> bool {
		self.rlp.is_empty()
	}

	/// List value
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.is_list());
	/// }
	/// ```
	pub fn is_list(&self) -> bool {
		self.rlp.is_list()
	}

	/// String value
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.at(1).is_data());
	/// }
	/// ```
	pub fn is_data(&self) -> bool {
		self.rlp.is_data()
	}

	/// Int value
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc1, 0x10];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert_eq!(rlp.is_int(), false);
	/// 	assert_eq!(rlp.at(0).is_int(), true);
	/// }
	/// ```
	pub fn is_int(&self) -> bool {
		self.rlp.is_int()
	}

	/// Get iterator over rlp-slices
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let strings: Vec<String> = rlp.iter().map(| i | i.as_val()).collect();
	/// }
	/// ```
	pub fn iter(&'view self) -> RlpIterator<'a, 'view> {
		self.into_iter()
	}

	/// Decode data into an object
	pub fn as_val<T>(&self) -> T where T: Decodable {
		self.rlp.as_val().expect("Unexpected rlp error")
	}

	pub fn as_list<T>(&self) -> Vec<T> where T: Decodable {
		self.iter().map(|rlp| rlp.as_val()).collect()
	}

	/// Decode data at given list index into an object
	pub fn val_at<T>(&self, index: usize) -> T where T: Decodable {
		self.at(index).as_val()
	}

	pub fn list_at<T>(&self, index: usize) -> Vec<T> where T: Decodable {
		self.at(index).as_list()
	}
}

/// Iterator over trusted rlp-slice list elements.
pub struct RlpIterator<'a, 'view> where 'a: 'view {
	rlp: &'view Rlp<'a>,
	index: usize
}

impl<'a, 'view> IntoIterator for &'view Rlp<'a> where 'a: 'view {
	type Item = Rlp<'a>;
	type IntoIter = RlpIterator<'a, 'view>;

	fn into_iter(self) -> Self::IntoIter {
		RlpIterator {
			rlp: self,
			index: 0,
		}
	}
}

impl<'a, 'view> Iterator for RlpIterator<'a, 'view> {
	type Item = Rlp<'a>;

	fn next(&mut self) -> Option<Rlp<'a>> {
		let index = self.index;
		let result = self.rlp.rlp.at(index).ok().map(From::from);
		self.index += 1;
		result
	}
}

#[test]
fn break_it() {
	use rustc_serialize::hex::FromHex;
	use bigint::prelude::U256;

	let h: Vec<u8> = FromHex::from_hex("f84d0589010efbef67941f79b2a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap();
	let r: Rlp = Rlp::new(&h);
	let u: U256 = r.val_at(1);
	assert_eq!(format!("{}", u), "19526463837540678066");
}
