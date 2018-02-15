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

use std::fmt;

use ethereum_types::{H256, U256, Address};
use ethcore_miner::pool;
use ethcore_miner::service_transaction_checker::{self, ServiceTransactionChecker};
use transaction::{
	self,
	UnverifiedTransaction,
	SignedTransaction,
};

use account_provider::AccountProvider;
use client::{MiningBlockChainClient, BlockId, TransactionId};
use engines::EthEngine;
use header::Header;

#[derive(Clone)]
pub struct BlockChainClient<'a> {
	chain: &'a MiningBlockChainClient,
	engine: &'a EthEngine,
	accounts: Option<&'a AccountProvider>,
	best_block_header: Header,
	service_transaction_checker: Option<ServiceTransactionChecker>,
}

impl<'a> BlockChainClient<'a> {
	pub fn new(
		chain: &'a MiningBlockChainClient,
		engine: &'a EthEngine,
		accounts: Option<&'a AccountProvider>,
		refuse_service_transactions: bool,
	) -> Self {
		let best_block_header = chain.best_block_header().decode();
		BlockChainClient {
			chain,
			engine,
			accounts,
			best_block_header,
			service_transaction_checker: if refuse_service_transactions {
				None
			} else {
				Some(Default::default())
			},
		}
	}

	pub fn verify_signed(&self, tx: &SignedTransaction) -> Result<(), transaction::Error> {
		self.engine.machine().verify_transaction(&tx, &self.best_block_header, self.chain.as_block_chain_client())
	}
}

impl<'a> fmt::Debug for BlockChainClient<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "BlockChainClient")
	}
}

impl<'a> pool::client::Client for BlockChainClient<'a> {
	fn transaction_already_included(&self, hash: &H256) -> bool {
		self.chain.transaction_block(TransactionId::Hash(*hash)).is_some()
	}

	fn verify_transaction(&self, tx: UnverifiedTransaction)
		-> Result<SignedTransaction, transaction::Error>
	{
		self.engine.verify_transaction_basic(&tx, &self.best_block_header)?;
		let tx = self.engine.verify_transaction_unordered(tx, &self.best_block_header)?;

		self.verify_signed(&tx)?;

		Ok(tx)
	}

	fn account_details(&self, address: &Address) -> pool::client::AccountDetails {
		pool::client::AccountDetails {
			nonce: self.chain.latest_nonce(address),
			balance: self.chain.latest_balance(address),
			is_local: self.accounts.map_or(false, |accounts| accounts.has_account(*address).unwrap_or(false)),
		}
	}

	fn account_nonce(&self, address: &Address) -> U256 {
		self.chain.latest_nonce(address)
	}

	fn required_gas(&self, tx: &SignedTransaction) -> U256 {
		tx.gas_required(&self.chain.latest_schedule()).into()
	}

	fn transaction_type(&self, tx: &SignedTransaction) -> pool::client::TransactionType {
		match self.service_transaction_checker {
			None => pool::client::TransactionType::Regular,
			Some(ref checker) => match checker.check(self, &tx.sender()) {
				Ok(true) => pool::client::TransactionType::Service,
				Ok(false) => pool::client::TransactionType::Regular,
				Err(e) => {
					debug!(target: "txqueue", "Unable to verify service transaction: {:?}", e);
					pool::client::TransactionType::Regular
				},
			}
		}
	}
}

impl<'a> service_transaction_checker::ContractCaller for BlockChainClient<'a> {
	fn registry_address(&self, name: &str) -> Option<Address> {
		self.chain.registry_address(name.into())
	}

	fn call_contract(&self, address: Address, data: Vec<u8>) -> Result<Vec<u8>, String> {
		self.chain.call_contract(BlockId::Latest, address, data)
	}
}
