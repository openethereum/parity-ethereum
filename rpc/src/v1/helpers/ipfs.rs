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

//! IPFS utility functions

use multihash;
use cid::{Cid, Codec, Version};
use crypto::digest;
use jsonrpc_core::Error;
use v1::types::Bytes;
use super::errors;

/// Compute CIDv0 from protobuf encoded bytes.
pub fn cid(content: Bytes) -> Result<String, Error> {
	let hash = digest::sha256(&content.0);
	let mh = multihash::encode(multihash::Hash::SHA2256, &*hash).map_err(errors::encoding)?;
	let cid = Cid::new(Codec::DagProtobuf, Version::V0, &mh);
	Ok(cid.to_string().into())
}
