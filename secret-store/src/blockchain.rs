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

use std::sync::Arc;
use bytes::Bytes;
use ethereum_types::{H256, Address, Public};
use ethabi::RawLog;
use crypto::publickey::{Signature, Error as EthKeyError};

/// Type for block number.
/// Duplicated from ethcore types
pub type BlockNumber = u64;

/// Uniquely identifies block.
/// Duplicated from ethcore types
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum BlockId {
	/// Block's sha3.
	/// Querying by hash is always faster.
	Hash(H256),
	/// Block number within canon blockchain.
	Number(BlockNumber),
	/// Earliest block (genesis).
	Earliest,
	/// Latest mined block.
	Latest,
}

/// Contract address.
#[derive(Debug, Clone)]
pub enum ContractAddress {
	/// Address is read from registry.
	Registry,
	/// Address is specified.
	Address(ethereum_types::Address),
}

/// Key pair with signing ability.
pub trait SigningKeyPair: Send + Sync {
	/// Public portion of key.
	fn public(&self) -> &Public;
	/// Address of key owner.
	fn address(&self) -> Address;
	/// Sign data with the key.
	fn sign(&self, data: &H256) -> Result<Signature, EthKeyError>;
}

/// Wrapps client ChainNotify in order to send signal about new blocks
pub trait NewBlocksNotify: Send + Sync {
	/// Fires when chain has new blocks.
	/// Sends this signal only, if contracts' update required
	fn new_blocks(&self, _new_enacted_len: usize) {
		// does nothing by default
	}
}

/// Blockchain logs Filter.
#[derive(Debug, PartialEq)]
pub struct Filter {
	/// Blockchain will be searched from this block.
	pub from_block: BlockId,

	/// Search addresses.
	///
	/// If None, match all.
	/// If specified, log must be produced by one of these addresses.
	pub address: Option<Vec<Address>>,

	/// Search topics.
	///
	/// If None, match all.
	/// If specified, log must contain one of these topics.
	pub topics: Vec<Option<Vec<H256>>>,
}

/// Blockchain representation for Secret Store
pub trait SecretStoreChain: Send + Sync + 'static {
	/// Adds listener for chain's NewBlocks event
	fn add_listener(&self, target: Arc<dyn NewBlocksNotify>);

	/// Check if the underlying chain is in the trusted state
	fn is_trusted(&self) -> bool;

	/// Transact contract.
	fn transact_contract(&self, contract: Address, tx_data: Bytes) -> Result<(), EthKeyError>;

	/// Read contract address. If address source is registry, address only returned if current client state is
	/// trusted. Address from registry is read from registry from block latest block with
	/// REQUEST_CONFIRMATIONS_REQUIRED confirmations.
	fn read_contract_address(&self, registry_name: &str, address: &ContractAddress) -> Option<Address>;

	/// Call contract in the blockchain
	fn call_contract(&self, block_id: BlockId, contract_address: Address, data: Bytes) -> Result<Bytes, String>;

	/// Returns blockhash for block id
	fn block_hash(&self, id: BlockId) -> Option<H256>;

	/// Returns block number for block id
	fn block_number(&self, id: BlockId) -> Option<BlockNumber>;

	/// Retrieve last blockchain logs for the filter
	fn retrieve_last_logs(&self, filter: Filter) -> Option<Vec<RawLog>>;

	/// Get hash of the last block with predefined number of confirmations (depends on the chain).
	fn get_confirmed_block_hash(&self) -> Option<H256>;
}
