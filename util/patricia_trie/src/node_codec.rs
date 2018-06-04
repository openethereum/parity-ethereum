//pub trait NodeCodec {
//	type N;
//	type O: Decodable + Debug;
//	fn encoded(&self) -> Vec<u8>;
//	fn decoded(data: &[u8]) -> Result<Self::N, Self::O>;
//	fn try_decode_hash(data: &[u8]) -> Option<Self::O>;
//}

use bytes::*;
use rlp::{DecoderError, Decodable};

pub trait NodeCodec<'a>: Sized {
	fn encoded(&self) -> Bytes;
	fn decoded(data: &'a [u8]) -> Result<Self, DecoderError>;
	fn try_decode_hash<O>(data: &[u8]) -> Option<O> where O: Decodable;
}

