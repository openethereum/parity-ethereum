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

use ethereum_types::Address;
use transaction::SignedTransaction;
use types::ids::BlockId;

use_contract!(service_transaction, "ServiceTransaction", "res/service_transaction.json");

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
	contract: service_transaction::ServiceTransaction,
}

impl ServiceTransactionChecker {
	/// Checks if service transaction can be appended to the transaction queue.
	pub fn check(&self, client: &ContractCaller, tx: &SignedTransaction) -> Result<bool, String> {
		assert!(tx.gas_price.is_zero());

		let address = client.registry_address(SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME)
			.ok_or_else(|| "contract is not configured")?;

		trace!(target: "txqueue", "Checking service transaction checker contract from {}", address);

		self.contract.functions()
			.certified()
			.call(tx.sender(), &|data| client.call_contract(BlockId::Latest, address, data))
			.map_err(|e| e.to_string())
	}
}
