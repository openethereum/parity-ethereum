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

//! A service transactions contract checker.

use std::collections::BTreeMap;
use client::{RegistryInfo, CallContract, BlockId, ChainInfo};
use types::transaction::SignedTransaction;
use ethabi::FunctionOutputDecoder;
use ethereum_types::Address;
use parking_lot::RwLock;

use_contract!(service_transaction, "res/contracts/service_transaction.json");

const SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME: &'static str = "service_transaction_checker";

/// Certified addresses cache
#[derive(Default)]
struct CertifiedAddressesCache {
	cache: RwLock<BTreeMap<Address, bool>>,
	last_block: u64
}

/// Service transactions checker.
#[derive(Default, Clone)]
pub struct ServiceTransactionChecker {
	certified_addresses_cache: CertifiedAddressesCache
}

impl ServiceTransactionChecker {
	/// Checks if given address in tx is whitelisted to send service transactions.
	pub fn check<C: CallContract + RegistryInfo>(&self, client: &C, tx: &SignedTransaction) -> Result<bool, String> {
		let sender = tx.sender();
		// Skip checking the contract if the transaction does not have zero gas price
		if !tx.gas_price.is_zero() {
			return Ok(false)
		}

		self.check_address(client, sender)
	}

	/// Checks if given address is whitelisted to send service transactions.
	pub fn check_address<C: CallContract + RegistryInfo>(&self, client: &C, sender: Address) -> Result<bool, String> {
		let contract_address = client.registry_address(SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest)
			.ok_or_else(|| "contract is not configured")?;
		trace!(target: "txqueue", "Checking service transaction checker contract from {}", sender);
		let (data, decoder) = service_transaction::functions::certified::call(sender);
		let value = client.call_contract(BlockId::Latest, contract_address, data)?;
		decoder.decode(&value).map_err(|e| e.to_string())
	/// Refresh certified addresses cache
	pub fn refresh_cache<C: CallContract + RegistryInfo + ChainInfo>(&mut self, client: &C) -> Result<bool, String> {
		trace!(target: "txqueue", "Refreshing certified addresses cache");
		let contract_address = client.registry_address(SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest)
			.ok_or_else(|| "contract is not configured")?;
		let mut updated_addresses: BTreeMap<Address, bool> = BTreeMap::default();
		let cache = self.certified_addresses_cache.cache.try_read();
		if cache.is_some() {
			for (address, allowed_before) in cache.unwrap().iter() {
				let (data, decoder) = service_transaction::functions::certified::call(*address);
				let value = client.call_contract(BlockId::Latest, contract_address, data)?;
				let allowed = decoder.decode(&value).map_err(|e| e.to_string())?;
				if *allowed_before != allowed {
					updated_addresses.insert(*address, allowed);
				}
			};
			let cache = self.certified_addresses_cache.cache.try_write();
			if cache.is_some() {
				let mut unwrapped_cache = cache.unwrap();
				for (address, allowed) in updated_addresses.iter() {
					unwrapped_cache.insert(*address, *allowed);
				}
				self.certified_addresses_cache.last_block = client.chain_info().best_block_number as u64;
				Ok(true)
			} else {
				Ok(false)
			}
		} else {
			Ok(false)
		}
	}
}

impl Clone for CertifiedAddressesCache {
	fn clone(&self) -> CertifiedAddressesCache {
		CertifiedAddressesCache {
			cache: RwLock::new(
				self.cache.try_read()
					.and_then(|c| Some(c.clone()))
					.or_else(|| Some(BTreeMap::default()))
					.unwrap()
			),
			last_block: self.last_block.clone()
		}
	}
}
