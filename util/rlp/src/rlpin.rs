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

use std::fmt;
use rustc_serialize::hex::ToHex;
use ::{View, DecoderError, UntrustedRlp, PayloadInfo, Prototype, RlpDecodable};

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

impl<'a, 'view> View<'a, 'view> for Rlp<'a> where 'a: 'view {
	type Prototype = Prototype;
	type PayloadInfo = PayloadInfo;
	type Data = &'a [u8];
	type Item = Rlp<'a>;
	type Iter = RlpIterator<'a, 'view>;

	/// Create a new instance of `Rlp`
	fn new(bytes: &'a [u8]) -> Rlp<'a> {
		Rlp {
			rlp: UntrustedRlp::new(bytes)
		}
	}

	fn as_raw(&'view self) -> &'a [u8] {
		self.rlp.as_raw()
	}

	fn prototype(&self) -> Self::Prototype {
		self.rlp.prototype().unwrap()
	}

	fn payload_info(&self) -> Self::PayloadInfo {
		self.rlp.payload_info().unwrap()
	}

	fn data(&'view self) -> Self::Data {
		self.rlp.data().unwrap()
	}

	fn item_count(&self) -> usize {
		self.rlp.item_count()
	}

	fn size(&self) -> usize {
		self.rlp.size()
	}

	fn at(&'view self, index: usize) -> Self::Item {
		From::from(self.rlp.at(index).unwrap())
	}

	fn is_null(&self) -> bool {
		self.rlp.is_null()
	}

	fn is_empty(&self) -> bool {
		self.rlp.is_empty()
	}

	fn is_list(&self) -> bool {
		self.rlp.is_list()
	}

	fn is_data(&self) -> bool {
		self.rlp.is_data()
	}

	fn is_int(&self) -> bool {
		self.rlp.is_int()
	}

	fn iter(&'view self) -> Self::Iter {
		self.into_iter()
	}

	fn as_val<T>(&self) -> Result<T, DecoderError> where T: RlpDecodable {
		self.rlp.as_val()
	}

	fn val_at<T>(&self, index: usize) -> Result<T, DecoderError> where T: RlpDecodable {
		self.at(index).rlp.as_val()
	}
}

impl <'a, 'view> Rlp<'a> where 'a: 'view {
	fn view_as_val<T, R>(r: &'view R) -> T where R: View<'a, 'view>, T: RlpDecodable {
		let res: Result<T, DecoderError> = r.as_val();
		res.unwrap_or_else(|e| panic!("DecodeError: {}, {}", e, r.as_raw().to_hex()))
	}

	/// Decode into an object
	pub fn as_val<T>(&self) -> T where T: RlpDecodable {
		Self::view_as_val(self)
	}

	/// Decode list item at given index into an object
	pub fn val_at<T>(&self, index: usize) -> T where T: RlpDecodable {
		Self::view_as_val(&self.at(index))
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
	use bigint::uint::U256;

	let h: Vec<u8> = FromHex::from_hex("f84d0589010efbef67941f79b2a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap();
	let r: Rlp = Rlp::new(&h);
	let u: U256 = r.val_at(1);
	assert_eq!(format!("{}", u), "19526463837540678066");
}
