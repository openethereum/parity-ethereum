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

use {rlp, multihash, IpfsHandler};
use error::{Error, Result};
use cid::{ToCid, Codec};

use multihash::Hash;
use ethereum_types::H256;
use bytes::Bytes;
use ethcore::client::{BlockId, TransactionId};

type Reason = &'static str;

/// Keeps the state of the response to send out
#[derive(Debug, PartialEq)]
pub enum Out {
	OctetStream(Bytes),
	NotFound(Reason),
	Bad(Reason),
}

impl IpfsHandler {
	/// Route path + query string to a specialized method
	pub fn route(&self, path: &str, query: Option<&str>) -> Out {
		match path {
			"/api/v0/block/get" => {
				let arg = query.and_then(|q| get_param(q, "arg")).unwrap_or("");

				self.route_cid(arg).unwrap_or_else(Into::into)
			},

			_ => Out::NotFound("Route not found")
		}
	}

	/// Attempt to read Content ID from `arg` query parameter, get a hash and
	/// route further by the CID's codec.
	fn route_cid(&self, cid: &str) -> Result<Out> {
		let cid = cid.to_cid()?;

		let mh = multihash::decode(&cid.hash)?;

		if mh.alg != Hash::Keccak256 { return Err(Error::UnsupportedHash); }

		let hash: H256 = mh.digest.into();

		match cid.codec {
			Codec::EthereumBlock => self.block(hash),
			Codec::EthereumBlockList => self.block_list(hash),
			Codec::EthereumTx => self.transaction(hash),
			Codec::EthereumStateTrie => self.state_trie(hash),
			Codec::Raw => self.contract_code(hash),
			_ => return Err(Error::UnsupportedCid),
		}
	}

	/// Get block header by hash as raw binary.
	fn block(&self, hash: H256) -> Result<Out> {
		let block_id = BlockId::Hash(hash);
		let block = self.client().block_header(block_id).ok_or(Error::BlockNotFound)?;

		Ok(Out::OctetStream(block.into_inner()))
	}

	/// Get list of block ommers by hash as raw binary.
	fn block_list(&self, hash: H256) -> Result<Out> {
		let uncles = self.client().find_uncles(&hash).ok_or(Error::BlockNotFound)?;

		Ok(Out::OctetStream(rlp::encode_list(&uncles).into_vec()))
	}

	/// Get transaction by hash and return as raw binary.
	fn transaction(&self, hash: H256) -> Result<Out> {
		let tx_id = TransactionId::Hash(hash);
		let tx = self.client().transaction(tx_id).ok_or(Error::TransactionNotFound)?;

		Ok(Out::OctetStream(rlp::encode(&*tx).into_vec()))
	}

	/// Get state trie node by hash and return as raw binary.
	fn state_trie(&self, hash: H256) -> Result<Out> {
		let data = self.client().state_data(&hash).ok_or(Error::StateRootNotFound)?;

		Ok(Out::OctetStream(data))
	}

	/// Get state trie node by hash and return as raw binary.
	fn contract_code(&self, hash: H256) -> Result<Out> {
		let data = self.client().state_data(&hash).ok_or(Error::ContractNotFound)?;

		Ok(Out::OctetStream(data))
	}
}

/// Get a query parameter's value by name.
fn get_param<'a>(query: &'a str, name: &str) -> Option<&'a str> {
	query.split('&')
		.find(|part| part.starts_with(name) && part[name.len()..].starts_with("="))
		.map(|part| &part[name.len() + 1..])
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use super::*;
	use ethcore::client::TestBlockChainClient;

	fn get_mocked_handler() -> IpfsHandler {
		IpfsHandler::new(None.into(), None.into(), Arc::new(TestBlockChainClient::new()))
	}

	#[test]
	fn test_get_param() {
		let query = "foo=100&bar=200&qux=300";

		assert_eq!(get_param(query, "foo"), Some("100"));
		assert_eq!(get_param(query, "bar"), Some("200"));
		assert_eq!(get_param(query, "qux"), Some("300"));
		assert_eq!(get_param(query, "bar="), None);
		assert_eq!(get_param(query, "200"), None);
		assert_eq!(get_param("", "foo"), None);
		assert_eq!(get_param("foo", "foo"), None);
		assert_eq!(get_param("foo&bar", "foo"), None);
		assert_eq!(get_param("bar&foo", "foo"), None);
	}

	#[test]
	fn cid_route_block() {
		let handler = get_mocked_handler();

		// `eth-block` with Keccak-256
		let cid = "z43AaGF5tmkT9SEX6urrhwpEW5ZSaACY73Vw357ZXTsur2fR8BM";

		assert_eq!(Err(Error::BlockNotFound), handler.route_cid(cid));
	}

	#[test]
	fn cid_route_block_list() {
		let handler = get_mocked_handler();

		// `eth-block-list` with Keccak-256
		let cid = "z43c7o7FsNxqdLJW8Ucj19tuCALtnmUb2EkDptj4W6xSkFVTqWs";

		assert_eq!(Err(Error::BlockNotFound), handler.route_cid(cid));
	}

	#[test]
	fn cid_route_tx() {
		let handler = get_mocked_handler();

		// `eth-tx` with Keccak-256
		let cid = "z44VCrqbpbPcb8SUBc8Tba4EaKuoDz2grdEoQXx4TP7WYh9ZGBu";

		assert_eq!(Err(Error::TransactionNotFound), handler.route_cid(cid));
	}

	#[test]
	fn cid_route_state_trie() {
		let handler = get_mocked_handler();

		// `eth-state-trie` with Keccak-256
		let cid = "z45oqTS7kR2n2peRGJQ4VCJEeaG9sorqcCyfmznZPJM7FMdhQCT";

		assert_eq!(Err(Error::StateRootNotFound), handler.route_cid(&cid));
	}

	#[test]
	fn cid_route_contract_code() {
		let handler = get_mocked_handler();

		// `raw` with Keccak-256
		let cid = "zb34WAp1Q5fhtLGZ3w3jhnTWaNbVV5ZZvGq4vuJQzERj6Pu3H";

		assert_eq!(Err(Error::ContractNotFound), handler.route_cid(&cid));
	}

	#[test]
	fn cid_route_invalid_hash() {
		let handler = get_mocked_handler();

		// `eth-block` with SHA3-256 hash
		let cid = "z43Aa9gr1MM7TENJh4Em9d9Ttr7p3UcfyMpNei6WLVeCmSEPu8F";

		assert_eq!(Err(Error::UnsupportedHash), handler.route_cid(cid));
	}

	#[test]
	fn cid_route_invalid_codec() {
		let handler = get_mocked_handler();

		// `bitcoin-block` with Keccak-256
		let cid = "z4HFyHvb8CarYARyxz4cCcPaciduXd49TFPCKLhYmvNxf7Auvwu";

		assert_eq!(Err(Error::UnsupportedCid), handler.route_cid(&cid));
	}

	#[test]
	fn route_block() {
		let handler = get_mocked_handler();

		let out = handler.route("/api/v0/block/get", Some("arg=z43AaGF5tmkT9SEX6urrhwpEW5ZSaACY73Vw357ZXTsur2fR8BM"));

		assert_eq!(out, Out::NotFound("Block not found"));
	}

	#[test]
	fn route_block_missing_query() {
		let handler = get_mocked_handler();

		let out = handler.route("/api/v0/block/get", None);

		assert_eq!(out, Out::Bad("CID parsing failed"));
	}

	#[test]
	fn route_block_invalid_query() {
		let handler = get_mocked_handler();

		let out = handler.route("/api/v0/block/get", Some("arg=foobarz43AaGF5tmkT9SEX6urrhwpEW5ZSaACY73Vw357ZXTsur2fR8BM"));

		assert_eq!(out, Out::Bad("CID parsing failed"));
	}

	#[test]
	fn route_invalid_route() {
		let handler = get_mocked_handler();

		let out = handler.route("/foo/bar/baz", Some("arg=z43AaGF5tmkT9SEX6urrhwpEW5ZSaACY73Vw357ZXTsur2fR8BM"));

		assert_eq!(out, Out::NotFound("Route not found"));
	}
}
