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

/// Validator set maintained in a contract which can be found in the registry under "validator_set".

use std::sync::Weak;
use lru_cache::LruCache;
use util::*;
use util::cache::MemoryLruCache;
use ethjson::hash::{Address as AddressSpec};
use ethjson::spec;
use types::ids::BlockId;
use client::{Client, BlockChainClient};
use super::ValidatorSet;
use super::new_validator_set;

/// Number of memoized contract validator sets.
const ADDRESS_SET_CAPACITY: usize = 3;
/// Nubmer of memoized validator set registry lookups.
const BLOCK_ADDRESS_SIZE: usize = 1_000_000;
const REGISTRY_NAME: &'static str  = "validator_set";

type ContractMaker = Box<Fn(Address) -> Box<ValidatorSet> + Send + Sync>;
type RegistryLookup = Box<Fn(H256) -> Address + Send + Sync>;

/// The validator contract should have the following interface:
pub struct RegistryContract {
	validators: RwLock<LruCache<Address, Box<ValidatorSet>>>,
	addresses: RwLock<MemoryLruCache<H256, Address>>,
	contract_maker: RwLock<ContractMaker>,
	registry_lookup: RwLock<RegistryLookup>,
	contract_type: spec::ValidatorContract,
}

impl RegistryContract {
	pub fn new(contract_type: spec::ValidatorContract) -> Self {
		RegistryContract {
			validators: RwLock::new(LruCache::new(ADDRESS_SET_CAPACITY)),
			addresses: RwLock::new(MemoryLruCache::new(BLOCK_ADDRESS_SIZE)),
			contract_maker: RwLock::new(Box::new(move |_| new_validator_set(spec::ValidatorSet::List(Default::default())))),
			registry_lookup: RwLock::new(Box::new(move |_| Default::default())),
			contract_type: contract_type,
		}
	}

	pub fn check<F, T>(&self, block_hash: &H256, query: F) -> T where F: Fn(&Box<ValidatorSet>) -> T {
		let mut addresses = self.addresses.write();
		let mut validators = self.validators.write();
		let maybe_address = addresses.get_mut(block_hash).map(|a| a.clone());
		let (maybe_answer, address) = match maybe_address {
			None => {
				let address = (self.registry_lookup.read())(block_hash.clone());
				let ans = validators.get_mut(&address).map(|s| query(&*s));
				addresses.insert(block_hash.clone(), address);
				(ans, address)
			},
			Some(a) => (validators.get_mut(&a).map(|s| query(&*s)), a),
		};
		maybe_answer.unwrap_or_else(|| {
			let set = (self.contract_maker.read())(address);
			let ans = query(&set);
			validators.insert(address, set);
			ans
		})
	}
}

impl ValidatorSet for RegistryContract {
	fn contains(&self, block_hash: &H256, address: &Address) -> bool {
		self.check(block_hash, |s| s.contains(block_hash, address))
	}

	fn get(&self, block_hash: &H256, nonce: usize) -> Address {
		self.check(block_hash, |s| s.get(block_hash, nonce))
	}

	fn count(&self, block_hash: &H256) -> usize {
		self.check(block_hash, |s| s.count(block_hash))
	}

	fn report_malicious(&self, address: &Address) {
		for (_, set) in self.validators.read().iter() {
			set.report_malicious(address)
		}
	}

	fn report_benign(&self, address: &Address) {
		for (_, set) in self.validators.read().iter() {
			set.report_benign(address)
		}
	}

	fn register_contract(&self, client: Weak<Client>) {
		let reg_client = client.clone();
		*self.registry_lookup.write() = Box::new(move |hash| reg_client
			.upgrade()
			.and_then(|c| c.registry_address(BlockId::Hash(hash), REGISTRY_NAME.into()))
			.unwrap_or_else(Default::default));
		let contract = self.contract_type;
		*self.contract_maker.write() = Box::new(move |address| {
			let set = new_validator_set(match contract {
				spec::ValidatorContract::Safe => spec::ValidatorSet::SafeContract(AddressSpec::from(address)),
				spec::ValidatorContract::Reporting => spec::ValidatorSet::ReportingContract(AddressSpec::from(address)),
			});
			set.register_contract(client.clone());
			set
		});
	}
}
