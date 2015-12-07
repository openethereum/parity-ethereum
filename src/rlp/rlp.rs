use super::faces::Reader;
use super::untrusted_rlp::*;

/// Data-oriented view onto trusted rlp-slice.
/// 
/// Unlikely to `UntrustedRlp` doesn't bother you with error
/// handling. It assumes that you know what you are doing.
pub struct Rlp<'a> {
	rlp: UntrustedRlp<'a>
}

impl<'a, 'view> Reader<'a, 'view> for Rlp<'a> where 'a: 'view {
	type Prototype = Prototype;
	type PayloadInfo = PayloadInfo;
	type Data = &'a [u8];
	type Item = Rlp<'a>;
	
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
		unimplemented!()
	}

	fn payload_info(&self) -> Self::PayloadInfo {
		unimplemented!()
	}

	fn data(&'view self) -> Self::Data {
		unimplemented!()
	}

	fn item_count(&self) -> usize {
		unimplemented!()
	}

	fn size(&self) -> usize {
		unimplemented!()
	}

	fn at(&'view self, index: usize) -> Self::Item {
		unimplemented!()
	}

	fn is_null(&self) -> bool {
		unimplemented!()
	}

	fn is_empty(&self) -> bool {
		unimplemented!()
	}

	fn is_list(&self) -> bool {
		unimplemented!()
	}

	fn is_data(&self) -> bool {
		unimplemented!()
	}

	fn is_int(&self) -> bool {
		unimplemented!()
	}
}
