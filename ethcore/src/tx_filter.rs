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

//! Smart contract based transaction filter.

use ethereum_types::{H256, U256, Address};
use lru_cache::LruCache;

use client::{BlockInfo, CallContract, BlockId};
use parking_lot::Mutex;
use spec::CommonParams;
use transaction::{Action, SignedTransaction};
use hash::KECCAK_EMPTY;

use_contract!(transact_acl_deprecated, "TransactAclDeprecated", "res/contracts/tx_acl_deprecated.json");
use_contract!(transact_acl, "TransactAcl", "res/contracts/tx_acl.json");

const MAX_CACHE_SIZE: usize = 4096;

mod tx_permissions {
	pub const _ALL: u32 = 0xffffffff;
	pub const NONE: u32 = 0x0;
	pub const BASIC: u32 = 0b00000001;
	pub const CALL: u32 = 0b00000010;
	pub const CREATE: u32 = 0b00000100;
	pub const _PRIVATE: u32 = 0b00001000;
}

/// Connection filter that uses a contract to manage permissions.
pub struct TransactionFilter {
	contract_deprecated: transact_acl_deprecated::TransactAclDeprecated,
	contract: transact_acl::TransactAcl,
	contract_address: Address,
	permission_cache: Mutex<LruCache<(H256, Address), u32>>,
	contract_version_cache: Mutex<LruCache<(H256), Option<U256>>>
}

impl TransactionFilter {
	/// Create a new instance if address is specified in params.
	pub fn from_params(params: &CommonParams) -> Option<TransactionFilter> {
		params.transaction_permission_contract.map(|address|
			TransactionFilter {
				contract_deprecated: transact_acl_deprecated::TransactAclDeprecated::default(),
				contract: transact_acl::TransactAcl::default(),
				contract_address: address,
				permission_cache: Mutex::new(LruCache::new(MAX_CACHE_SIZE)),
				contract_version_cache: Mutex::new(LruCache::new(MAX_CACHE_SIZE)),
			}
		)
	}

	/// Check if transaction is allowed at given block.
	pub fn transaction_allowed<C: BlockInfo + CallContract>(&self, parent_hash: &H256, transaction: &SignedTransaction, client: &C) -> bool {
		let mut permission_cache = self.permission_cache.lock();
		let mut contract_version_cache = self.contract_version_cache.lock();

		let (tx_type, to) = match transaction.action {
			Action::Create => (tx_permissions::CREATE, Address::new()),
			Action::Call(address) => if client.code_hash(&address, BlockId::Hash(*parent_hash)).map_or(false, |c| c != KECCAK_EMPTY) {
					(tx_permissions::CALL, address)
				} else {
					(tx_permissions::BASIC, address)
				}
		};

		let sender = transaction.sender();
		let value = transaction.value;
		let key = (*parent_hash, sender);

		if let Some(permissions) = permission_cache.get_mut(&key) {
			return *permissions & tx_type != 0;
		}

		let contract_address = self.contract_address;
		let contract_version = contract_version_cache.get_mut(parent_hash).and_then(|v| *v).or_else(||  {
			self.contract.functions()
				.contract_version()
				.call(&|data| client.call_contract(BlockId::Hash(*parent_hash), contract_address, data))
				.ok()
		});
		contract_version_cache.insert(*parent_hash, contract_version);

		// Check permissions in smart contract based on its version
		let (permissions, filter_only_sender) = match contract_version {
			Some(version) => {
				let version_u64 = version.low_u64();
				trace!(target: "tx_filter", "Version of tx permission contract: {}", version);
				match version_u64 {
					2 => self.contract.functions()
						.allowed_tx_types()
						.call(sender, to, value, &|data| client.call_contract(BlockId::Hash(*parent_hash), contract_address, data))
						.map(|(p, f)| (p.low_u32(), f))
						.unwrap_or_else(|e| {
							error!(target: "tx_filter", "Error calling tx permissions contract: {:?}", e);
							(tx_permissions::NONE, true)
						}),
					_ => {
						error!(target: "tx_filter", "Unknown version of tx permissions contract is used");
						(tx_permissions::NONE, true)
					}
				}
			},
			None => {
				trace!(target: "tx_filter", "Fallback to the deprecated version of tx permission contract");
				(self.contract_deprecated.functions()
					 .allowed_tx_types()
					 .call(sender, &|data| client.call_contract(BlockId::Hash(*parent_hash), contract_address, data))
					 .map(|p| p.low_u32())
					 .unwrap_or_else(|e| {
						 error!(target: "tx_filter", "Error calling tx permissions contract: {:?}", e);
						 tx_permissions::NONE
					 }), true)
			}
		};

		if filter_only_sender {
			permission_cache.insert((*parent_hash, sender), permissions);
		}
		trace!(target: "tx_filter",
			"Given transaction data: sender: {:?} to: {:?} value: {}. Permissions required: {:X}, got: {:X}",
			   sender, to, value, tx_type, permissions
		);
		permissions & tx_type != 0
	}
}

#[cfg(test)]
mod test {
	use std::sync::Arc;
	use spec::Spec;
	use client::{BlockChainClient, Client, ClientConfig, BlockId};
	use miner::Miner;
	use ethereum_types::{U256, Address};
	use io::IoChannel;
	use ethkey::{Secret, KeyPair};
	use super::TransactionFilter;
	use transaction::{Transaction, Action};
	use tempdir::TempDir;

	/// Contract code: https://gist.github.com/VladLupashevskyi/84f18eabb1e4afadf572cf92af3e7e7f
	#[test]
	fn transaction_filter() {
		let spec_data = include_str!("../res/tx_permission_tests/contract_ver_2_genesis.json");

		let tempdir = TempDir::new("").unwrap();
		let spec = Spec::load(&tempdir.path(), spec_data.as_bytes()).unwrap();
		let client_db = Arc::new(::kvdb_memorydb::create(::db::NUM_COLUMNS.unwrap_or(0)));

		let client = Client::new(
			ClientConfig::default(),
			&spec,
			client_db,
			Arc::new(Miner::new_for_tests(&spec, None)),
			IoChannel::disconnected(),
		).unwrap();
		let key1 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000001")).unwrap();
		let key2 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000002")).unwrap();
		let key3 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000003")).unwrap();
		let key4 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000004")).unwrap();
		let key5 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000005")).unwrap();
		let key6 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000006")).unwrap();
		let key7 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000007")).unwrap();

		let filter = TransactionFilter::from_params(spec.params()).unwrap();
		let mut basic_tx = Transaction::default();
		basic_tx.action = Action::Call(Address::from("d41c057fd1c78805aac12b0a94a405c0461a6fbb"));
		let create_tx = Transaction::default();
		let mut call_tx = Transaction::default();
		call_tx.action = Action::Call(Address::from("0000000000000000000000000000000000000005"));

		let mut basic_tx_with_ether_and_to_key7 = Transaction::default();
		basic_tx_with_ether_and_to_key7.action = Action::Call(Address::from("d41c057fd1c78805aac12b0a94a405c0461a6fbb"));
		basic_tx_with_ether_and_to_key7.value = U256::from(123123);
		let mut call_tx_with_ether = Transaction::default();
		call_tx_with_ether.action = Action::Call(Address::from("0000000000000000000000000000000000000005"));
		call_tx_with_ether.value = U256::from(123123);

		let mut basic_tx_to_key6 = Transaction::default();
		basic_tx_to_key6.action = Action::Call(Address::from("e57bfe9f44b819898f47bf37e5af72a0783e1141"));
		let mut basic_tx_with_ether_and_to_key6 = Transaction::default();
		basic_tx_with_ether_and_to_key6.action = Action::Call(Address::from("e57bfe9f44b819898f47bf37e5af72a0783e1141"));
		basic_tx_with_ether_and_to_key6.value = U256::from(123123);

		let genesis = client.block_hash(BlockId::Latest).unwrap();

		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key1.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &create_tx.clone().sign(key1.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &call_tx.clone().sign(key1.secret(), None), &*client));

		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key2.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &create_tx.clone().sign(key2.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &call_tx.clone().sign(key2.secret(), None), &*client));

		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key3.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &create_tx.clone().sign(key3.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &call_tx.clone().sign(key3.secret(), None), &*client));

		assert!(!filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key4.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &create_tx.clone().sign(key4.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &call_tx.clone().sign(key4.secret(), None), &*client));

		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key1.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &create_tx.clone().sign(key1.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &call_tx.clone().sign(key1.secret(), None), &*client));

		assert!(!filter.transaction_allowed(&genesis, &basic_tx_with_ether_and_to_key7.clone().sign(key5.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &call_tx_with_ether.clone().sign(key5.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key6.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &basic_tx_with_ether_and_to_key7.clone().sign(key6.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &basic_tx_to_key6.clone().sign(key7.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &basic_tx_with_ether_and_to_key6.clone().sign(key7.secret(), None), &*client));
	}

	/// Contract code: https://gist.github.com/arkpar/38a87cb50165b7e683585eec71acb05a
	#[test]
	fn transaction_filter_deprecated() {
		let spec_data = include_str!("../res/tx_permission_tests/deprecated_contract_genesis.json");

		let tempdir = TempDir::new("").unwrap();
		let spec = Spec::load(&tempdir.path(), spec_data.as_bytes()).unwrap();
		let client_db = Arc::new(::kvdb_memorydb::create(::db::NUM_COLUMNS.unwrap_or(0)));

		let client = Client::new(
			ClientConfig::default(),
			&spec,
			client_db,
			Arc::new(Miner::new_for_tests(&spec, None)),
			IoChannel::disconnected(),
		).unwrap();
		let key1 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000001")).unwrap();
		let key2 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000002")).unwrap();
		let key3 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000003")).unwrap();
		let key4 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000004")).unwrap();

		let filter = TransactionFilter::from_params(spec.params()).unwrap();
		let mut basic_tx = Transaction::default();
		basic_tx.action = Action::Call(Address::from("000000000000000000000000000000000000032"));
		let create_tx = Transaction::default();
		let mut call_tx = Transaction::default();
		call_tx.action = Action::Call(Address::from("0000000000000000000000000000000000000005"));

		let genesis = client.block_hash(BlockId::Latest).unwrap();

		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key1.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &create_tx.clone().sign(key1.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &call_tx.clone().sign(key1.secret(), None), &*client));

		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key2.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &create_tx.clone().sign(key2.secret(), None), &*client));
		assert!(filter.transaction_allowed(&genesis, &call_tx.clone().sign(key2.secret(), None), &*client));

		assert!(filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key3.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &create_tx.clone().sign(key3.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &call_tx.clone().sign(key3.secret(), None), &*client));

		assert!(!filter.transaction_allowed(&genesis, &basic_tx.clone().sign(key4.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &create_tx.clone().sign(key4.secret(), None), &*client));
		assert!(!filter.transaction_allowed(&genesis, &call_tx.clone().sign(key4.secret(), None), &*client));
	}
}
