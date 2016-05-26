// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::collections::HashSet;
use types::transaction_request::TransactionRequest;

pub trait SigningQueue {
	fn add_request(&mut self, transaction: TransactionRequest);

	fn remove_request(&mut self, id: TransactionRequest);

	fn requests(&self) -> &HashSet<TransactionRequest>;
}

impl SigningQueue for HashSet<TransactionRequest> {
	fn add_request(&mut self, transaction: TransactionRequest) {
		self.insert(transaction);
	}

	fn remove_request(&mut self, id: TransactionRequest) {
		self.remove(&id);
	}

	fn requests(&self) -> &HashSet<TransactionRequest> {
		self
	}
}


#[cfg(test)]
mod test {
	use std::collections::HashSet;
	use util::hash::Address;
	use util::numbers::U256;
	use types::transaction_request::TransactionRequest;
	use super::*;

	#[test]
	fn should_work_for_hashset() {
		// given
		let mut queue = HashSet::new();

		let request = TransactionRequest {
			from: Address::from(1),
			to: Some(Address::from(2)),
			gas_price: None,
			gas: None,
			value: Some(U256::from(10_000_000)),
			data: None,
			nonce: None,
		};

		// when
		queue.add_request(request.clone());
		let all = queue.requests();

		// then
		assert_eq!(all.len(), 1);
		assert!(all.contains(&request));
	}
}
