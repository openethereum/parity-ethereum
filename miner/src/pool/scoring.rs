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

//! Transaction Scoring and Ordering
//!
//! Ethereum transactions from the same sender are ordered by `nonce`.
//! Low nonces need to be included first. If there are two transactions from the same sender
//! and with the same `nonce` only one of them can be included.
//! We choose the one with higher gas price, but also require that gas price increment
//! is high enough to prevent attacking miners by requiring them to reshuffle/reexecute
//! the queue too often.
//!
//! Transactions between senders are prioritized using `gas price`. Higher `gas price`
//! yields more profits for miners. Additionally we prioritize transactions that originate
//! from our local node (own transactions).

use std::cmp;
use std::sync::Arc;

use ethereum_types::U256;
use txpool;
use super::VerifiedTransaction;

/// Transaction with the same (sender, nonce) can be replaced only if
/// `new_gas_price > old_gas_price + old_gas_price >> SHIFT`
const GAS_PRICE_BUMP_SHIFT: usize = 3; // 2 = 25%, 3 = 12.5%, 4 = 6.25%


/// Simple, gas-price based scoring for transactions.
///
/// TODO [ToDr] Consider including:
/// - Penalization
/// - Score of first transaction = Max/Avg(gas price of all transactions)
#[derive(Debug)]
pub struct GasPrice;

impl txpool::Scoring<VerifiedTransaction> for GasPrice {
	type Score = U256;

	fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering {
		// TODO [ToDr] Handle null-sender transactions
		old.transaction.nonce.cmp(&other.transaction.nonce)
	}

	fn choose(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> txpool::scoring::Choice {
		if old.transaction.nonce != new.transaction.nonce {
			return txpool::scoring::Choice::InsertNew
		}

		let old_gp = old.transaction.gas_price;
		let new_gp = new.transaction.gas_price;

		let min_required_gp = old_gp + (old_gp >> GAS_PRICE_BUMP_SHIFT);
		// TODO [ToDr] Handle null-sender transactions
		match min_required_gp.cmp(&new_gp) {
			cmp::Ordering::Greater => txpool::scoring::Choice::RejectNew,
			_ => txpool::scoring::Choice::ReplaceOld,
		}
	}

	fn update_scores(&self, txs: &[Arc<VerifiedTransaction>], scores: &mut [U256], _change: txpool::scoring::Change) {
		// TODO [ToDr] Optimize
		for i in 0..txs.len() {
			scores[i] = txs[i].transaction.gas_price;
			let boost = match txs[i].priority() {
				super::Priority::Local => 10,
				super::Priority::Retracted => 5,
				super::Priority::Regular => 0,
			};
			// TODO [ToDr] overflow?
			scores[i] = scores[i] + scores[i] >> boost;
		}
	}

	fn should_replace(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> bool {
		self.choose(old, new) == txpool::scoring::Choice::ReplaceOld
	}
}
