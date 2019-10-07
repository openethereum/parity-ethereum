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
use call_contract::CallContract;
use registrar::RegistrarClient;
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
	pub fn check<C: CallContract + RegistrarClient>(
		&self,
		client: &C,
		tx: &SignedTransaction
	) -> Result<bool, String> {
		let sender = tx.sender();
		// Skip checking the contract if the transaction does not have zero gas price
		if !tx.gas_price.is_zero() {
			return Ok(false)
		}

		self.check_address(client, sender)
	}

	/// Checks if given address is whitelisted to send service transactions.
	pub fn check_address<C>(&self, client: &C, sender: Address) -> Result<bool, String>
		where C: CallContract + RegistrarClient
	{
		trace!(target: "txqueue", "Checking service transaction checker contract from {}", sender);
		if let Some(allowed) = self
			.certified_addresses_cache
			.try_read()
			.as_ref()
			.and_then(|c| c.get(&sender))
		{
			return Ok(*allowed);
		}

		let contract_address = match client.get_address(
			SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME,
			BlockId::Latest
		) {
			Ok(Some(addr)) => addr,
			Ok(None) => return Err("contract is not configured".to_owned()),
			Err(e) => return Err(e)
		};

		self.call_contract(client, contract_address, sender).and_then(|allowed| {
			if let Some(mut cache) = self.certified_addresses_cache.try_write() {
				cache.insert(sender, allowed);
			};
			Ok(allowed)
		})
	}

	/// Refresh certified addresses cache
	pub fn refresh_cache<C>(&self, client: &C) -> Result<bool, String>
		where C: CallContract + RegistrarClient
	{
		trace!(target: "txqueue", "Refreshing certified addresses cache");
		// replace the cache with an empty list,
		// since it's not recent it won't be used anyway.
		let cache = mem::replace(&mut *self.certified_addresses_cache.write(), HashMap::default());

		if client.registrar_address().is_none() {
			return Ok(false);
		}

		let contract_address_fetch = client.get_address(
			SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME,
			BlockId::Latest
		)?;

		if let Some(contract_address) = contract_address_fetch {
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

	fn call_contract<C>(
		&self,
		client: &C,
		contract_address: Address,
		sender: Address
	) -> Result<bool, String>
		where C: CallContract + RegistrarClient
	{
		let (data, decoder) = service_transaction::functions::certified::call(sender);
		let value = client.call_contract(BlockId::Latest, contract_address, data)?;
		decoder.decode(&value).map_err(|e| e.to_string())
	}
}
