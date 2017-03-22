// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Common RLP traits
use elastic_array::ElasticArray1024;
use {DecoderError, UntrustedRlp, RlpStream};

/// RLP decodable trait
pub trait Decodable: Sized {
	/// Decode a value from RLP bytes
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError>;
}

/// Structure encodable to RLP
pub trait Encodable {
	/// Append a value to the stream
	fn rlp_append(&self, s: &mut RlpStream);

	/// Get rlp-encoded bytes for this instance
	fn rlp_bytes(&self) -> ElasticArray1024<u8> {
		let mut s = RlpStream::new();
		self.rlp_append(&mut s);
		s.drain()
	}
}

/// Trait for compressing and decompressing RLP by replacement of common terms.
pub trait Compressible: Sized {
	/// Indicates the origin of RLP to be compressed.
	type DataType;

	/// Compress given RLP type using appropriate methods.
	fn compress(&self, t: Self::DataType) -> ElasticArray1024<u8>;
	/// Decompress given RLP type using appropriate methods.
	fn decompress(&self, t: Self::DataType) -> ElasticArray1024<u8>;
}
