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
use util::RwLock;

/// Storage for private transactions
pub struct PrivateTransactions {
	transactions: RwLock<Vec<UnverifiedTransaction>>,
}

impl PrivateTransactions {
	pub fn new() -> Self {
		PrivateTransactions {
			transactions: RwLock::new(Vec::new()),
		}
	}

	pub fn import(&self, transaction: UnverifiedTransaction, _peer_id: usize) -> Result<(), Error> {
		{
			let mut transactions = self.transactions.write();
			transactions.push(transaction.clone());
		}
		Ok(())
	}

	pub fn get_list(&self) -> Vec<UnverifiedTransaction> {
		let transactions = self.transactions.read();
		transactions.clone()
	}
}