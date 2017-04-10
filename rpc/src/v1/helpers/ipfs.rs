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

//! IPFS utility functions

use multihash;
use cid::{Cid, Codec, Version};
use rust_crypto::sha2::Sha256;
use rust_crypto::digest::Digest;
use jsonrpc_core::Error;
use v1::types::Bytes;
use super::errors;

/// Compute CIDv0 from protobuf encoded bytes.
pub fn cid(content: Bytes) -> Result<String, Error> {
	let mut hasher = Sha256::new();
	hasher.input(&content.0);
	let len = hasher.output_bytes();
	let mut buf = Vec::with_capacity(len);
	buf.resize(len, 0);
	hasher.result(&mut buf);
	let mh = multihash::encode(multihash::Hash::SHA2256, &buf).map_err(errors::encoding_error)?;
	let cid = Cid::new(Codec::DagProtobuf, Version::V0, &mh);
	Ok(cid.to_string().into())
}
