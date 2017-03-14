use byteorder::{ByteOrder, BigEndian};
use bigint::prelude::{Uint, U128, U256, H64, H128, H160, H256, H512, H520, H2048};
use traits::Encodable;
use stream::RlpStream;

impl Encodable for bool {
	fn rlp_append(&self, s: &mut RlpStream) {
		if *self {
			s.encoder().encode_value(&[1]);
		} else {
			s.encoder().encode_value(&[0]);
		}
	}
}

impl<'a> Encodable for &'a [u8] {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self);
	}
}

impl Encodable for Vec<u8> {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self);
	}
}

impl<T> Encodable for Option<T> where T: Encodable {
	fn rlp_append(&self, s: &mut RlpStream) {
		match *self {
			None => {
				s.begin_list(0);
			},
			Some(ref value) => {
				s.begin_list(1);
				s.append(value);
			}
		}
	}
}

impl Encodable for u8 {
	fn rlp_append(&self, s: &mut RlpStream) {
		if *self != 0 {
			s.encoder().encode_value(&[*self]);
		} else {
			s.encoder().encode_value(&[]);
		}
	}
}

macro_rules! impl_encodable_for_u {
	($name: ident, $func: ident, $size: expr) => {
		impl Encodable for $name {
			fn rlp_append(&self, s: &mut RlpStream) {
				let leading_empty_bytes = self.leading_zeros() as usize / 8;
				let mut buffer = [0u8; $size];
				BigEndian::$func(&mut buffer, *self);
				s.encoder().encode_value(&buffer[leading_empty_bytes..]);
			}
		}
	}
}

impl_encodable_for_u!(u16, write_u16, 2);
impl_encodable_for_u!(u32, write_u32, 4);
impl_encodable_for_u!(u64, write_u64, 8);

impl Encodable for usize {
	fn rlp_append(&self, s: &mut RlpStream) {
		(*self as u64).rlp_append(s);
	}
}

macro_rules! impl_encodable_for_hash {
	($name: ident) => {
		impl Encodable for $name {
			fn rlp_append(&self, s: &mut RlpStream) {
				s.encoder().encode_value(self);
			}
		}
	}
}

impl_encodable_for_hash!(H64);
impl_encodable_for_hash!(H128);
impl_encodable_for_hash!(H160);
impl_encodable_for_hash!(H256);
impl_encodable_for_hash!(H512);
impl_encodable_for_hash!(H520);
impl_encodable_for_hash!(H2048);

macro_rules! impl_encodable_for_uint {
	($name: ident, $size: expr) => {
		impl Encodable for $name {
			fn rlp_append(&self, s: &mut RlpStream) {
				let leading_empty_bytes = $size - (self.bits() + 7) / 8;
				let mut buffer = [0u8; $size];
				self.to_big_endian(&mut buffer);
				s.encoder().encode_value(&buffer[leading_empty_bytes..]);
			}
		}
	}
}

impl_encodable_for_uint!(U256, 32);
impl_encodable_for_uint!(U128, 16);

impl<'a> Encodable for &'a str {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self.as_bytes());
	}
}

impl Encodable for String {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self.as_bytes());
	}
}

