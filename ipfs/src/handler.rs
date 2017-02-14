use {rlp, multihash};
use error::{Error, Result};
use cid::{ToCid, Codec};

use std::sync::Arc;
use std::ops::Deref;
use multihash::Hash;
use hyper::Next;
use util::{Bytes, H256};
use ethcore::client::{BlockId, TransactionId, BlockChainClient};

type Reason = &'static str;

pub enum Out {
	OctetStream(Bytes),
	NotFound(Reason),
	Bad(Reason),
}

pub struct IpfsHandler {
	client: Arc<BlockChainClient>,
	out: Out,
}

impl IpfsHandler {
	pub fn new(client: Arc<BlockChainClient>) -> Self {
		IpfsHandler {
			client: client,
			out: Out::NotFound("Route not found")
		}
	}

	pub fn out(&self) -> &Out {
		&self.out
	}

	pub fn route(&mut self, path: &str, query: Option<&str>) -> Next {
		let result = match path {
			"/api/v0/block/get" => self.route_cid(query),
			_ => return Next::write(),
		};

		match result {
			Ok(_) => Next::write(),
			Err(err) => {
				self.out = err.into();

				Next::write()
			}
		}
	}

	fn route_cid(&mut self, query: Option<&str>) -> Result<()> {
		let query = query.unwrap_or("");

		let cid = get_param(&query, "arg").ok_or(Error::CidParsingFailed)?.to_cid()?;

		let mh = multihash::decode(&cid.hash)?;

		if mh.alg != Hash::Keccak256 { return Err(Error::UnsupportedHash); }

		let hash: H256 = mh.digest.into();

		match cid.codec {
			Codec::EthereumBlock => self.get_block(hash),
			Codec::EthereumBlockList => self.get_block_list(hash),
			Codec::EthereumTx => self.get_transaction(hash),
			Codec::EthereumStateTrie => self.get_state_trie(hash),
			_ => return Err(Error::UnsupportedCid),
		}
	}

	fn get_block(&mut self, hash: H256) -> Result<()> {
		let block_id = BlockId::Hash(hash);
		let block = self.client.block_header(block_id).ok_or(Error::BlockNotFound)?;

		self.out = Out::OctetStream(block.into_inner());

		Ok(())
	}

	fn get_block_list(&mut self, hash: H256) -> Result<()> {
		let ommers = self.client.find_uncles(&hash).ok_or(Error::BlockNotFound)?;

		self.out = Out::OctetStream(rlp::encode(&ommers).to_vec());

		Ok(())
	}

	fn get_transaction(&mut self, hash: H256) -> Result<()> {
		let tx_id = TransactionId::Hash(hash);
		let tx = self.client.transaction(tx_id).ok_or(Error::TransactionNotFound)?;

		self.out = Out::OctetStream(rlp::encode(tx.deref()).to_vec());

		Ok(())
	}

	fn get_state_trie(&mut self, hash: H256) -> Result<()> {
		let data = self.client.state_data(&hash).ok_or(Error::StateRootNotFound)?;

		self.out = Out::OctetStream(data);

		Ok(())
	}
}

/// Get a query parameter's value by name.
pub fn get_param<'a>(query: &'a str, name: &str) -> Option<&'a str> {
	query.split('&')
		.find(|part| part.starts_with(name) && part[name.len()..].starts_with("="))
		.map(|part| &part[name.len() + 1..])
}

#[cfg(test)]
mod tests {
	use super::*;

   #[test]
	fn test_get_param() {
		let query = "foo=100&bar=200&qux=300";

		assert_eq!(get_param(query, "foo"), Some("100"));
		assert_eq!(get_param(query, "bar"), Some("200"));
		assert_eq!(get_param(query, "qux"), Some("300"));
		assert_eq!(get_param(query, "bar="), None);
		assert_eq!(get_param(query, "200"), None);
	}
}
