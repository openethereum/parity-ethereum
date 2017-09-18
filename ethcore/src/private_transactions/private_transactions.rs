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

use transaction::UnverifiedTransaction;
use error::Error;
use parking_lot::RwLock;

/// Storage for private transactions
pub struct PrivateTransactions {
	transactions: RwLock<Vec<UnverifiedTransaction>>,
	signed_transactions:  RwLock<Vec<UnverifiedTransaction>>,
}

impl PrivateTransactions {
	pub fn new() -> Self {
		PrivateTransactions {
			transactions: RwLock::new(Vec::new()),
			signed_transactions: RwLock::new(Vec::new()),
		}
	}

	pub fn import_transaction(&self, transaction: UnverifiedTransaction, _peer_id: usize) -> Result<(), Error> {
		let mut transactions = self.transactions.write();
		transactions.push(transaction);
		Ok(())
	}

	pub fn import_signed_transaction(&self, transaction: UnverifiedTransaction, _peer_id: usize) -> Result<(), Error> {
		let mut signed_transactions = self.signed_transactions.write();
		signed_transactions.push(transaction);
		Ok(())
	}

	pub fn get_transactions_list(&self) -> Vec<UnverifiedTransaction> {
		self.transactions.read().clone()
	}

	pub fn get_signed_transactions_list(&self) -> Vec<UnverifiedTransaction> {
		self.signed_transactions.read().clone()
	}
}