use bytes::BytesConvertable;
// use hash::FixedHash;

pub trait Bloomable {
	fn shift_bloom<T>(&mut self, bytes: &T) where T: BytesConvertable;
	fn contains_bloom<T>(&self, bytes: &T) -> bool where T: BytesConvertable;
}
