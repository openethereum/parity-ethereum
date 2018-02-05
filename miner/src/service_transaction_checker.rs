// Copyright 2017 Parity Technologies (UK) Ltd.
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
// along with Parity. If not, see <http://www.gnu.org/licenses/>.

//! A service transactions contract checker.

use futures::{future, Future};
use native_contracts::ServiceTransactionChecker as Contract;
use ethereum_types::{U256, Address};
use parking_lot::Mutex;
use transaction::SignedTransaction;
use types::ids::BlockId;

const SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME: &'static str = "service_transaction_checker";

/// A contract calling interface.
pub trait ContractCaller {
	/// Returns address of contract from the registry, given it's name
	fn registry_address(&self, name: &str) -> Option<Address>;

	/// Executes a contract call at given block.
	fn call_contract(&self, BlockId, Address, Vec<u8>) -> Result<Vec<u8>, String>;
}

/// Service transactions checker.
#[derive(Default)]
pub struct ServiceTransactionChecker {
	contract: Mutex<Option<Contract>>,
}

impl ServiceTransactionChecker {
	/// Try to create instance, reading contract address from given chain client.
	pub fn update_from_chain_client(&self, client: &ContractCaller) {
		let mut contract = self.contract.lock();
		if contract.is_none() {
			*contract = client.registry_address(SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME)
				.and_then(|contract_addr| {
					trace!(target: "txqueue", "Configuring for service transaction checker contract from {}", contract_addr);

					Some(Contract::new(contract_addr))
				})
		}
	}

	/// Checks if service transaction can be appended to the transaction queue.
	pub fn check(&self, client: &ContractCaller, tx: &SignedTransaction) -> Result<bool, String> {
		debug_assert_eq!(tx.gas_price, U256::zero());

		if let Some(ref contract) = *self.contract.lock() {
			contract.certified(
				|addr, data| future::done(client.call_contract(BlockId::Latest, addr, data)),
				tx.sender()
			).wait()
		} else {
			Err("contract is not configured".to_owned())
		}
	}
}
