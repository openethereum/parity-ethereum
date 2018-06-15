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

/// Trait with common encoding operations
// TODO: document this
pub trait Stream {
	fn new() -> Self;
	fn new_list(len: usize) -> Self;
	fn append_empty_data(&mut self) -> &mut Self;
	fn drain(self) -> ElasticArray1024<u8>; // TODO: add as assoc type? Makes the types kind of hairy and requires some extra trait bounds, but not sure if it's worth it. Needs AsRef<u8> I think.
	fn append_bytes<'a>(&'a mut self, bytes: &[u8]) -> &'a mut Self;
	fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut Self;
}