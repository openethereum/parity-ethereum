// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::ops::Deref;
use std::collections::BTreeMap;

use ethereum_types::{H160, H256, U256, Bloom as H2048};
use serde::ser::Error;
use serde::{Serialize, Serializer};
use types::encoded::Header as EthHeader;
use v1::types::{Bytes, Transaction};

/// Block Transactions
#[derive(Debug)]
pub enum BlockTransactions {
	/// Only hashes
	Hashes(Vec<H256>),
	/// Full transactions
	Full(Vec<Transaction>)
}

impl Serialize for BlockTransactions {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			BlockTransactions::Hashes(ref hashes) => hashes.serialize(serializer),
			BlockTransactions::Full(ref ts) => ts.serialize(serializer)
		}
	}
}

/// Block representation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
	/// Hash of the block
	pub hash: Option<H256>,
	/// Hash of the parent
	pub parent_hash: H256,
	/// Hash of the uncles
	#[serde(rename = "sha3Uncles")]
	pub uncles_hash: H256,
	/// Authors address
	pub author: H160,
	/// Alias of `author`
	pub miner: H160,
	/// State root hash
	pub state_root: H256,
	/// Transactions root hash
	pub transactions_root: H256,
	/// Transactions receipts root hash
	pub receipts_root: H256,
	/// Block number
	pub number: Option<U256>,
	/// Gas Used
	pub gas_used: U256,
	/// Gas Limit
	pub gas_limit: U256,
	/// Extra data
	pub extra_data: Bytes,
	/// Logs bloom
	pub logs_bloom: Option<H2048>,
	/// Timestamp
	pub timestamp: U256,
	/// Difficulty
	pub difficulty: U256,
	/// Total difficulty
	pub total_difficulty: Option<U256>,
	/// Seal fields
	pub seal_fields: Vec<Bytes>,
	/// Uncles' hashes
	pub uncles: Vec<H256>,
	/// Transactions
	pub transactions: BlockTransactions,
	/// Size in bytes
	pub size: Option<U256>,
}

/// Block header representation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Header {
	/// Hash of the block
	pub hash: Option<H256>,
	/// Hash of the parent
	pub parent_hash: H256,
	/// Hash of the uncles
	#[serde(rename = "sha3Uncles")]
	pub uncles_hash: H256,
	/// Authors address
	pub author: H160,
	/// Alias of `author`
	pub miner: H160,
	/// State root hash
	pub state_root: H256,
	/// Transactions root hash
	pub transactions_root: H256,
	/// Transactions receipts root hash
	pub receipts_root: H256,
	/// Block number
	pub number: Option<U256>,
	/// Gas Used
	pub gas_used: U256,
	/// Gas Limit
	pub gas_limit: U256,
	/// Extra data
	pub extra_data: Bytes,
	/// Logs bloom
	pub logs_bloom: H2048,
	/// Timestamp
	pub timestamp: U256,
	/// Difficulty
	pub difficulty: U256,
	/// Seal fields
	pub seal_fields: Vec<Bytes>,
	/// Size in bytes
	pub size: Option<U256>,
}

impl From<EthHeader> for Header {
	fn from(h: EthHeader) -> Self {
		(&h).into()
	}
}

impl<'a> From<&'a EthHeader> for Header {
	fn from(h: &'a EthHeader) -> Self {
		Header {
			hash: Some(h.hash()),
			size: Some(h.rlp().as_raw().len().into()),
			parent_hash: h.parent_hash(),
			uncles_hash: h.uncles_hash(),
			author: h.author(),
			miner: h.author(),
			state_root: h.state_root(),
			transactions_root: h.transactions_root(),
			receipts_root: h.receipts_root(),
			number: Some(h.number().into()),
			gas_used: h.gas_used(),
			gas_limit: h.gas_limit(),
			logs_bloom: h.log_bloom(),
			timestamp: h.timestamp().into(),
			difficulty: h.difficulty(),
			extra_data: h.extra_data().into(),
			seal_fields: h.view().decode_seal()
				.expect("Client/Miner returns only valid headers. We only serialize headers from Client/Miner; qed")
				.into_iter().map(Into::into).collect(),
		}
	}
}

/// Block representation with additional info.
pub type RichBlock = Rich<Block>;

/// Header representation with additional info.
pub type RichHeader = Rich<Header>;

/// Value representation with additional info
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rich<T> {
	/// Standard value.
	pub inner: T,
	/// Engine-specific fields with additional description.
	/// Should be included directly to serialized block object.
	// TODO [ToDr] #[serde(skip_serializing)]
	pub extra_info: BTreeMap<String, String>,
}

impl<T> Deref for Rich<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T: Serialize> Serialize for Rich<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		use serde_json::{to_value, Value};

		let serialized = (to_value(&self.inner), to_value(&self.extra_info));
		if let (Ok(Value::Object(mut value)), Ok(Value::Object(extras))) = serialized {
			// join two objects
			value.extend(extras);
			// and serialize
			value.serialize(serializer)
		} else {
			Err(S::Error::custom("Unserializable structures: expected objects"))
		}
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use ethereum_types::{H64, H160, H256, U256, Bloom as H2048};
	use serde_json;
	use v1::types::{Transaction, Bytes};
	use super::{Block, RichBlock, BlockTransactions, Header, RichHeader};

	#[test]
	fn test_serialize_block_transactions() {
		let t = BlockTransactions::Full(vec![Transaction::default()]);
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"[{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x0000000000000000000000000000000000000000","to":null,"value":"0x0","gasPrice":"0x0","gas":"0x0","input":"0x","creates":null,"raw":"0x","publicKey":null,"chainId":null,"standardV":"0x0","v":"0x0","r":"0x0","s":"0x0","condition":null}]"#);

		let t = BlockTransactions::Hashes(vec![H256::zero().into()]);
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"["0x0000000000000000000000000000000000000000000000000000000000000000"]"#);
	}

	#[test]
	fn test_serialize_block() {
		let block = Block {
			hash: Some(H256::zero()),
			parent_hash: H256::zero(),
			uncles_hash: H256::zero(),
			author: H160::default(),
			miner: H160::default(),
			state_root: H256::zero(),
			transactions_root: H256::zero(),
			receipts_root: H256::zero(),
			number: Some(U256::default()),
			gas_used: U256::default(),
			gas_limit: U256::default(),
			extra_data: Bytes::default(),
			logs_bloom: Some(H2048::default()),
			timestamp: U256::default(),
			difficulty: U256::default(),
			total_difficulty: Some(U256::default()),
			seal_fields: vec![Bytes::default(), Bytes::default()],
			uncles: vec![],
			transactions: BlockTransactions::Hashes(vec![].into()),
			size: Some(69.into()),
		};
		let serialized_block = serde_json::to_string(&block).unwrap();
		let rich_block = RichBlock {
			inner: block,
			extra_info: map![
				"mixHash".into() => format!("{:?}", H256::zero()),
				"nonce".into() => format!("{:?}", H64::default())
			],
		};
		let serialized_rich_block = serde_json::to_string(&rich_block).unwrap();

		assert_eq!(serialized_block, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","author":"0x0000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","number":"0x0","gasUsed":"0x0","gasLimit":"0x0","extraData":"0x","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","difficulty":"0x0","totalDifficulty":"0x0","sealFields":["0x","0x"],"uncles":[],"transactions":[],"size":"0x45"}"#);
		assert_eq!(serialized_rich_block, r#"{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x0","extraData":"0x","gasLimit":"0x0","gasUsed":"0x0","hash":"0x0000000000000000000000000000000000000000000000000000000000000000","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","number":"0x0","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","sealFields":["0x","0x"],"sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","size":"0x45","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","totalDifficulty":"0x0","transactions":[],"transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","uncles":[]}"#);
	}

	#[test]
	fn none_size_null() {
		let block = Block {
			hash: Some(H256::zero()),
			parent_hash: H256::zero(),
			uncles_hash: H256::zero(),
			author: H160::default(),
			miner: H160::default(),
			state_root: H256::zero(),
			transactions_root: H256::zero(),
			receipts_root: H256::zero(),
			number: Some(U256::default()),
			gas_used: U256::default(),
			gas_limit: U256::default(),
			extra_data: Bytes::default(),
			logs_bloom: Some(H2048::default()),
			timestamp: U256::default(),
			difficulty: U256::default(),
			total_difficulty: Some(U256::default()),
			seal_fields: vec![Bytes::default(), Bytes::default()],
			uncles: vec![],
			transactions: BlockTransactions::Hashes(vec![].into()),
			size: None,
		};
		let serialized_block = serde_json::to_string(&block).unwrap();
		let rich_block = RichBlock {
			inner: block,
			extra_info: map![
				"mixHash".into() => format!("{:?}", H256::zero()),
				"nonce".into() => format!("{:?}", H64::default())
			],
		};
		let serialized_rich_block = serde_json::to_string(&rich_block).unwrap();

		assert_eq!(serialized_block, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","author":"0x0000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","number":"0x0","gasUsed":"0x0","gasLimit":"0x0","extraData":"0x","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","difficulty":"0x0","totalDifficulty":"0x0","sealFields":["0x","0x"],"uncles":[],"transactions":[],"size":null}"#);
		assert_eq!(serialized_rich_block, r#"{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x0","extraData":"0x","gasLimit":"0x0","gasUsed":"0x0","hash":"0x0000000000000000000000000000000000000000000000000000000000000000","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","number":"0x0","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","sealFields":["0x","0x"],"sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","size":null,"stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","totalDifficulty":"0x0","transactions":[],"transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","uncles":[]}"#);
	}

	#[test]
	fn test_serialize_header() {
		let header = Header {
			hash: Some(H256::zero()),
			parent_hash: H256::zero(),
			uncles_hash: H256::zero(),
			author: H160::default(),
			miner: H160::default(),
			state_root: H256::zero(),
			transactions_root: H256::zero(),
			receipts_root: H256::zero(),
			number: Some(U256::default()),
			gas_used: U256::default(),
			gas_limit: U256::default(),
			extra_data: Bytes::default(),
			logs_bloom: H2048::default(),
			timestamp: U256::default(),
			difficulty: U256::default(),
			seal_fields: vec![Bytes::default(), Bytes::default()],
			size: Some(69.into()),
		};
		let serialized_header = serde_json::to_string(&header).unwrap();
		let rich_header = RichHeader {
			inner: header,
			extra_info: map![
				"mixHash".into() => format!("{:?}", H256::zero()),
				"nonce".into() => format!("{:?}", H64::default())
			],
		};
		let serialized_rich_header = serde_json::to_string(&rich_header).unwrap();

		assert_eq!(serialized_header, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","author":"0x0000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","number":"0x0","gasUsed":"0x0","gasLimit":"0x0","extraData":"0x","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","difficulty":"0x0","sealFields":["0x","0x"],"size":"0x45"}"#);
		assert_eq!(serialized_rich_header, r#"{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x0","extraData":"0x","gasLimit":"0x0","gasUsed":"0x0","hash":"0x0000000000000000000000000000000000000000000000000000000000000000","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","number":"0x0","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","sealFields":["0x","0x"],"sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","size":"0x45","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#);
	}
}
