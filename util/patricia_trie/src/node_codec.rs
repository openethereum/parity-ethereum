// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Generic trait for trie node encoding/decoding. Takes a `hashdb::Hasher` 
//! to parametrize the hashes used in the codec; takes a `stream_encoder::Stream` 
//! implementation to do streaming encoding.

use bytes::Bytes;
use hashdb::Hasher;
use node::Node;
use stream_encoder::Stream;
use super::triedbmut::{ChildReference, NodeHandle}; // TODO: tidy this up

use elastic_array::{ElasticArray1024, ElasticArray128, ElasticArray36};

/// Trait for trie node encoding/decoding
pub trait NodeCodec<H: Hasher>: Sized {
	/// Encoding error type
	type E: ::std::error::Error;

	/// Encoded stream type
	type S: Stream;

	/// Null node type
	const HASHED_NULL_NODE: H::Out;
	
	/// Encode a Node to bytes (aka `Vec<u8>`).
	fn encode(&Node) -> Bytes;

	/// Decode bytes to a `Node`. Returns `Self::E` on failure.
	fn decode(data: &[u8]) -> Result<Node, Self::E>;

	/// Decode bytes to the `Hasher`s output type. Assumes 32 bytes long hashes! Returns `None` on failure.
	fn try_decode_hash(data: &[u8]) -> Option<H::Out>;

	// Check if the provided bytes correspond to the codecs "empty" node.
	fn is_empty_node(data: &[u8]) -> bool;

	
	fn empty_node() -> ElasticArray1024<u8>;
	fn leaf_node(partial: &[u8], value: &[u8]) -> ElasticArray1024<u8>;

    // fn ext_node<F>(partial: &[u8], child: NodeHandle<H>, cb: F) -> ElasticArray1024<u8> 
    fn ext_node<F>(partial: ElasticArray36<u8>, child: NodeHandle<H>, cb: F) -> ElasticArray1024<u8> 
	where F: FnMut(NodeHandle<H>) -> ChildReference<H::Out>;

	fn branch_node<I>(children: I, value: Option<ElasticArray128<u8>>) -> ElasticArray1024<u8>
	where 
		I: IntoIterator<Item=Option<ChildReference<H::Out>>>;
}
