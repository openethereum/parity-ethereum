extern crate hashdb;
extern crate ethcore_bytes;
extern crate elastic_array;

use hashdb::Hasher;
use ethcore_bytes::Bytes;
use elastic_array::ElasticArray1024;

pub trait Stream {
	fn new() -> Self;
	fn new_list(len: usize) -> Self;
	fn append_empty_data(&mut self) -> &mut Self;
	fn drain(self) -> ElasticArray1024<u8>; // TODO: add as assoc type? Makes the types kind of hairy and requires some extra trait bounds, but not sure if it's worth it. Needs AsRef<u8> I think.
	fn append_bytes<'a>(&'a mut self, bytes: &[u8]) -> &'a mut Self;
	fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut Self;
}

pub trait NodeCodec<H: Hasher>: Sized {
	type E: ::std::error::Error;
	type S: Stream;
    type Node;
	fn encode(Self::Node) -> Bytes;
	fn decode(data: &[u8]) -> Result<Self::Node, Self::E>;
	fn try_decode_hash(data: &[u8]) -> Option<H::Out>;

	fn is_empty_node(data: &[u8]) -> bool;
}