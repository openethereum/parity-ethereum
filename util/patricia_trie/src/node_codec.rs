use bytes::*;
use rlp::{DecoderError, Decodable};

pub trait NodeCodec<'a>: Sized {
	type Encoding;
	type StreamEncoding;

	fn encoded(&self) -> Bytes;
	fn decoded(data: &'a [u8]) -> Result<Self, DecoderError>;
	fn try_decode_hash<O>(data: &[u8]) -> Option<O> where O: Decodable;

	fn new_encoded(data: &'a[u8]) -> Self::Encoding;
	fn encoded_stream() -> Self::StreamEncoding;
	fn encoded_list(size: usize) -> Self::StreamEncoding;
}

