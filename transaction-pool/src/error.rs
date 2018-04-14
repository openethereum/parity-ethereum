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

use ethereum_types::H256;

error_chain! {
	errors {
		/// Transaction is already imported
		AlreadyImported(hash: H256) {
			description("transaction is already in the pool"),
			display("[{:?}] already imported", hash)
		}
		/// Transaction is too cheap to enter the queue
		TooCheapToEnter(hash: H256, min_score: String) {
			description("the pool is full and transaction is too cheap to replace any transaction"),
			display("[{:?}] too cheap to enter the pool. Min score: {}", hash, min_score)
		}
		/// Transaction is too cheap to replace existing transaction that occupies the same slot.
		TooCheapToReplace(old_hash: H256, hash: H256) {
			description("transaction is too cheap to replace existing transaction in the pool"),
			display("[{:?}] too cheap to replace: {:?}", hash, old_hash)
		}
	}
}

#[cfg(test)]
impl PartialEq for ErrorKind {
	fn eq(&self, other: &Self) -> bool {
		use self::ErrorKind::*;

		match (self, other) {
			(&AlreadyImported(ref h1), &AlreadyImported(ref h2)) => h1 == h2,
			(&TooCheapToEnter(ref h1, ref s1), &TooCheapToEnter(ref h2, ref s2)) => h1 == h2 && s1 == s2,
			(&TooCheapToReplace(ref old1, ref new1), &TooCheapToReplace(ref old2, ref new2)) => old1 == old2 && new1 == new2,
			_ => false,
		}
	}
}
