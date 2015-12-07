use super::faces::{View, Decodable};
use super::untrusted_rlp::*;

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

impl<'a, 'view> View<'a, 'view> for Rlp<'a> where 'a: 'view {
	type Prototype = Prototype;
	type PayloadInfo = PayloadInfo;
	type Data = &'a [u8];
	type Item = Rlp<'a>;
	type Iter = RlpIterator<'a, 'view>;
	type Error = DecoderError;
	
	/// Create a new instance of `Rlp`
	fn new(bytes: &'a [u8]) -> Rlp<'a> {
		Rlp {
			rlp: UntrustedRlp::new(bytes)
		}
	}

	fn raw(&'view self) -> &'a [u8] {
		self.rlp.raw()
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

	fn as_val<T>(&self) -> Result<T, Self::Error> where T: Decodable {
		self.rlp.as_val()
	}
}

impl <'a, 'view> Rlp<'a> where 'a: 'view {
	fn reader_as_val<T, R>(r: &R) -> T where R: View<'a, 'view>, T: Decodable {
		let res: Result<T, R::Error> = r.as_val();
		res.unwrap_or_else(|_| panic!())
	}

	pub fn as_val<T>(&self) -> T where T: Decodable {
		Self::reader_as_val(self)
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
		let result = self.rlp.rlp.at(index).ok().map(| iter | { From::from(iter) });
		self.index += 1;
		result
	}
}
