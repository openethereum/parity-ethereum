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

use std::collections::HashMap;
use std::mem;
use std::sync::Arc;
use call_contract::{RegistryInfo, CallContract};
use types::ids::BlockId;
use types::transaction::SignedTransaction;
use ethabi::FunctionOutputDecoder;
use ethereum_types::Address;
use parking_lot::RwLock;

use_contract!(service_transaction, "res/contracts/service_transaction.json");

const SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME: &'static str = "service_transaction_checker";

/// Service transactions checker.
#[derive(Default, Clone)]
pub struct ServiceTransactionChecker {
	certified_addresses_cache: Arc<RwLock<HashMap<Address, bool>>>
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
		trace!(target: "txqueue", "Checking service transaction checker contract from {}", sender);
		if let Some(allowed) = self.certified_addresses_cache.try_read().as_ref().and_then(|c| c.get(&sender)) {
			return Ok(*allowed);
		}
		let contract_address = client.registry_address(SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest)
			.ok_or_else(|| "contract is not configured")?;
		self.call_contract(client, contract_address, sender).and_then(|allowed| {
			if let Some(mut cache) = self.certified_addresses_cache.try_write() {
				cache.insert(sender, allowed);
			};
			Ok(allowed)
		})
	}

	/// Refresh certified addresses cache
	pub fn refresh_cache<C: CallContract + RegistryInfo>(&self, client: &C) -> Result<bool, String> {
		trace!(target: "txqueue", "Refreshing certified addresses cache");
		// replace the cache with an empty list,
		// since it's not recent it won't be used anyway.
		let cache = mem::replace(&mut *self.certified_addresses_cache.write(), HashMap::default());

		if let Some(contract_address) = client.registry_address(SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest) {
			let addresses: Vec<_> = cache.keys().collect();
			let mut cache: HashMap<Address, bool> = HashMap::default();
			for address in addresses {
				let allowed = self.call_contract(client, contract_address, *address)?;
				cache.insert(*address, allowed);
			}
			mem::replace(&mut *self.certified_addresses_cache.write(),  cache);
			Ok(true)
		} else {
			Ok(false)
		}
	}

	fn call_contract<C: CallContract + RegistryInfo>(&self, client: &C, contract_address: Address, sender: Address) -> Result<bool, String> {
		let (data, decoder) = service_transaction::functions::certified::call(sender);
		let value = client.call_contract(BlockId::Latest, contract_address, data)?;
		decoder.decode(&value).map_err(|e| e.to_string())
	}
}
