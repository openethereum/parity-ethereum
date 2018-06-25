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

//! Trait for encoded streams of bytes, used by RLP and other encoding schemes
extern crate elastic_array;

use elastic_array::ElasticArray1024;

/// Trait with common stream encoding operations
pub trait Stream {
	/// New instance of an empty stream
	fn new() -> Self;
	/// New instance of an empty stream, initialized as a "list"
	fn new_list(len: usize) -> Self;
	/// Append the encoding schemes "null" value. Chainable.
	fn append_empty_data(&mut self) -> &mut Self;
	/// Drain the object and return the underlying data
	fn drain(self) -> ElasticArray1024<u8>; // TODO: add as assoc type? Makes the types kind of hairy and requires some extra trait bounds, but not sure if it's worth it. Needs AsRef<u8> I think.
	/// Append the provided bytes to the end of the stream as a single item. Chainable.
	fn append_bytes<'a>(&'a mut self, bytes: &[u8]) -> &'a mut Self;
	/// Append pre-serialized data. Use with caution. Chainable.
	fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut Self;
}