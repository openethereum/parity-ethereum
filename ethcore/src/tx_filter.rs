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
	permission_cache: Mutex<LruCache<(H256, Address, Address, U256), u32>>,
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
			}
		)
	}

	/// Check if transaction is allowed at given block.
	pub fn transaction_allowed<C: BlockInfo + CallContract>(&self, parent_hash: &H256, transaction: &SignedTransaction, client: &C) -> bool {
		let mut cache = self.permission_cache.lock();
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
		let key = (*parent_hash, sender, to, value);

		if let Some(permissions) = cache.get_mut(&key) {
			return *permissions & tx_type != 0;
		}

		let contract_address = self.contract_address;

		// Check permissions in smart contracts
		let permissions = self.contract.functions()
			.allowed_tx_types()
			.call(sender, to, value, &|data| client.call_contract(BlockId::Hash(*parent_hash), contract_address, data))
			.map(|p| p.low_u32())
			.unwrap_or_else(|_e| {
				// If failed, first check deprecated contract
				trace!(target: "tx_filter", "Fallback to the deprecated version of tx permission contract");
				self.contract_deprecated.functions()
					.allowed_tx_types()
					.call(sender, &|data| client.call_contract(BlockId::Hash(*parent_hash), contract_address, data))
					.map(|p| p.low_u32())
					.unwrap_or_else(|e| {
						error!(target: "tx_filter", "Error calling tx permissions contract: {:?}", e);
						tx_permissions::NONE
					})
			});

		cache.insert((*parent_hash, sender, to, value), permissions);
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

	/// Contract code: https://gist.github.com/VladLupashevskyi/a157d8162db85de67de522eeb7c8ee85
	#[test]
	fn transaction_filter() {
		let spec_data = r#"
		{
			"name": "TestNodeFilterContract",
			"engine": {
				"authorityRound": {
					"params": {
						"stepDuration": 1,
						"startStep": 2,
						"validators": {
							"contract": "0x0000000000000000000000000000000000000000"
						}
					}
				}
			},
			"params": {
				"accountStartNonce": "0x0",
				"maximumExtraDataSize": "0x20",
				"minGasLimit": "0x1388",
				"networkID" : "0x69",
				"gasLimitBoundDivisor": "0x0400",
				"transactionPermissionContract": "0x0000000000000000000000000000000000000005"
			},
			"genesis": {
				"seal": {
					"generic": "0xc180"
				},
				"difficulty": "0x20000",
				"author": "0x0000000000000000000000000000000000000000",
				"timestamp": "0x00",
				"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"extraData": "0x",
				"gasLimit": "0x222222"
			},
			"accounts": {
				"0000000000000000000000000000000000000001": { "balance": "1", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
				"0000000000000000000000000000000000000002": { "balance": "1", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
				"0000000000000000000000000000000000000003": { "balance": "1", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
				"0000000000000000000000000000000000000004": { "balance": "1", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
				"0000000000000000000000000000000000000005": {
					"balance": "1",
					"constructor": "608060405234801561001057600080fd5b506104c3806100206000396000f300608060405260043610610062576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063469ab1e31461006757806375d0c0dc1461009a578063a0a8e4601461012a578063d4b03ee014610155575b600080fd5b34801561007357600080fd5b5061007c6101e2565b60405180826000191660001916815260200191505060405180910390f35b3480156100a657600080fd5b506100af610253565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156100ef5780820151818401526020810190506100d4565b50505050905090810190601f16801561011c5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34801561013657600080fd5b5061013f610290565b6040518082815260200191505060405180910390f35b34801561016157600080fd5b506101c0600480360381019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610299565b604051808263ffffffff1663ffffffff16815260200191505060405180910390f35b60006101ec610253565b6040518082805190602001908083835b60208310151561022157805182526020820191506020810190506020830392506101fc565b6001836020036101000a0380198251168184511680821785525050505050509050019150506040518091039020905090565b60606040805190810160405280601681526020017f54585f5045524d495353494f4e5f434f4e545241435400000000000000000000815250905090565b60006002905090565b6000737e5f4552091a69125d5dfcb7b8c2659029395bdf8473ffffffffffffffffffffffffffffffffffffffff1614156102d95763ffffffff9050610490565b732b5ad5c4795c026514f8317c7a215e218dccd6cf8473ffffffffffffffffffffffffffffffffffffffff1614156103175760026001179050610490565b736813eb9362372eef6200f3b1dbc3f819671cba698473ffffffffffffffffffffffffffffffffffffffff1614156103525760019050610490565b73e1ab8145f7e55dc933d51a18c793f901a3a0b2768473ffffffffffffffffffffffffffffffffffffffff1614801561038b5750600082145b1561039c5763ffffffff9050610490565b73e57bfe9f44b819898f47bf37e5af72a0783e11418473ffffffffffffffffffffffffffffffffffffffff161480156103fe575073d41c057fd1c78805aac12b0a94a405c0461a6fbb8373ffffffffffffffffffffffffffffffffffffffff16145b1561040c5760019050610490565b73d41c057fd1c78805aac12b0a94a405c0461a6fbb8473ffffffffffffffffffffffffffffffffffffffff1614801561046e575073e57bfe9f44b819898f47bf37e5af72a0783e11418373ffffffffffffffffffffffffffffffffffffffff16145b801561047a5750600082145b1561048b5763ffffffff9050610490565b600090505b93925050505600a165627a7a72305820ea35352692061875f689bdefc5e0d0ec86be33cdcf729bb0908f76b1da038b560029"
				}
			}
		}
		"#;

		let tempdir = TempDir::new("").unwrap();
		let spec = Spec::load(&tempdir.path(), spec_data.as_bytes()).unwrap();
		let client_db = Arc::new(::kvdb_memorydb::create(::db::NUM_COLUMNS.unwrap_or(0)));

		let client = Client::new(
			ClientConfig::default(),
			&spec,
			client_db,
			Arc::new(Miner::with_spec(&spec)),
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
		let spec_data = r#"
		{
			"name": "TestNodeFilterContract",
			"engine": {
				"authorityRound": {
					"params": {
						"stepDuration": 1,
						"startStep": 2,
						"validators": {
							"contract": "0x0000000000000000000000000000000000000000"
						}
					}
				}
			},
			"params": {
				"accountStartNonce": "0x0",
				"maximumExtraDataSize": "0x20",
				"minGasLimit": "0x1388",
				"networkID" : "0x69",
				"gasLimitBoundDivisor": "0x0400",
				"transactionPermissionContract": "0x0000000000000000000000000000000000000005"
			},
			"genesis": {
				"seal": {
					"generic": "0xc180"
				},
				"difficulty": "0x20000",
				"author": "0x0000000000000000000000000000000000000000",
				"timestamp": "0x00",
				"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"extraData": "0x",
				"gasLimit": "0x222222"
			},
			"accounts": {
				"0000000000000000000000000000000000000001": { "balance": "1", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
				"0000000000000000000000000000000000000002": { "balance": "1", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
				"0000000000000000000000000000000000000003": { "balance": "1", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
				"0000000000000000000000000000000000000004": { "balance": "1", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
				"0000000000000000000000000000000000000005": {
					"balance": "1",
					"constructor": "6060604052341561000f57600080fd5b5b6101868061001f6000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063e17512211461003e575b600080fd5b341561004957600080fd5b610075600480803573ffffffffffffffffffffffffffffffffffffffff16906020019091905050610097565b604051808263ffffffff1663ffffffff16815260200191505060405180910390f35b6000737e5f4552091a69125d5dfcb7b8c2659029395bdf8273ffffffffffffffffffffffffffffffffffffffff1614156100d75763ffffffff9050610155565b732b5ad5c4795c026514f8317c7a215e218dccd6cf8273ffffffffffffffffffffffffffffffffffffffff1614156101155760026001179050610155565b736813eb9362372eef6200f3b1dbc3f819671cba698273ffffffffffffffffffffffffffffffffffffffff1614156101505760019050610155565b600090505b9190505600a165627a7a72305820f1f21cb978925a8a92c6e30c8c81adf598adff6d1ef941cf5ed6c0ec7ad1ae3d0029"
				}
			}
		}
		"#;

		let tempdir = TempDir::new("").unwrap();
		let spec = Spec::load(&tempdir.path(), spec_data.as_bytes()).unwrap();
		let client_db = Arc::new(::kvdb_memorydb::create(::db::NUM_COLUMNS.unwrap_or(0)));

		let client = Client::new(
			ClientConfig::default(),
			&spec,
			client_db,
			Arc::new(Miner::with_spec(&spec)),
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

