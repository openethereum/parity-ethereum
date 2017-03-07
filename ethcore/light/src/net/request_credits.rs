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

//! Request credit management.
//!
//! Every request in the light protocol leads to a reduction
//! of the requester's amount of credits as a rate-limiting mechanism.
//! The amount of credits will recharge at a set rate.
//!
//! This module provides an interface for configuration of
//! costs and recharge rates of request credits.
//!
//! Current default costs are picked completely arbitrarily, not based
//! on any empirical timings or mathematical models.

use request::{self, Request};
use super::packet;
use super::error::Error;

use rlp::*;
use util::U256;
use time::{Duration, SteadyTime};

/// Credits value.
///
/// Produced and recharged using `FlowParams`.
/// Definitive updates can be made as well -- these will reset the recharge
/// point to the time of the update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credits {
	estimate: U256,
	recharge_point: SteadyTime,
}

impl Credits {
	/// Get the current amount of credits..
	pub fn current(&self) -> U256 { self.estimate.clone() }

	/// Make a definitive update.
	/// This will be the value obtained after receiving
	/// a response to a request.
	pub fn update_to(&mut self, value: U256) {
		self.estimate = value;
		self.recharge_point = SteadyTime::now();
	}

	/// Attempt to apply the given cost to the amount of credits.
	///
	/// If successful, the cost will be deducted successfully.
	///
	/// If unsuccessful, the structure will be unaltered an an
	/// error will be produced.
	pub fn deduct_cost(&mut self, cost: U256) -> Result<(), Error> {
		match cost > self.estimate {
			true => Err(Error::NoCredits),
			false => {
				self.estimate = self.estimate - cost;
				Ok(())
			}
		}
	}
}

/// A cost table, mapping requests to base and per-request costs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostTable {
	base: U256, // cost per packet.
	headers: U256, // cost per header
	body: U256,
	receipts: U256,
	account: U256,
	storage: U256,
	code: U256,
	header_proof: U256,
	transaction_proof: U256, // cost per gas.
}

impl Default for CostTable {
	fn default() -> Self {
		// arbitrarily chosen constants.
		CostTable {
			base: 100000.into(),
			headers: 10000.into(),
			body: 15000.into(),
			receipts: 5000.into(),
			account: 25000.into(),
			storage: 25000.into(),
			code: 20000.into(),
			header_proof: 15000.into(),
			transaction_proof: 2.into(),
		}
	}
}

impl RlpEncodable for CostTable {
	fn rlp_append(&self, s: &mut RlpStream) {
		unimplemented!()
	}
}

impl RlpDecodable for CostTable {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		unimplemented!()
	}
}

/// Handles costs, recharge, limits of request credits.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowParams {
	costs: CostTable,
	limit: U256,
	recharge: U256,
}

impl FlowParams {
	/// Create new flow parameters from a request cost table,
	/// credit limit, and (minimum) rate of recharge.
	pub fn new(limit: U256, costs: CostTable, recharge: U256) -> Self {
		FlowParams {
			costs: costs,
			limit: limit,
			recharge: recharge,
		}
	}

	/// Create effectively infinite flow params.
	pub fn free() -> Self {
		let free_cost: U256 = 0.into();
		FlowParams {
			limit: (!0u64).into(),
			recharge: 1.into(),
			costs: CostTable {
				base: free_cost.clone(),
				headers: free_cost.clone(),
				body: free_cost.clone(),
				receipts: free_cost.clone(),
				account: free_cost.clone(),
				storage: free_cost.clone(),
				code: free_cost.clone(),
				header_proof: free_cost.clone(),
				transaction_proof: free_cost,
			}
		}
	}

	/// Get a reference to the credit limit.
	pub fn limit(&self) -> &U256 { &self.limit }

	/// Get a reference to the cost table.
	pub fn cost_table(&self) -> &CostTable { &self.costs }

	/// Get a reference to the recharge rate.
	pub fn recharge_rate(&self) -> &U256 { &self.recharge }

	/// Compute the actual cost of a request, given the kind of request
	/// and number of requests made.
	pub fn compute_cost(&self, request: &Request) -> U256 {
		match *request {
			Request::Headers(ref req) => self.costs.headers * req.max.into(),
			Request::HeaderProof(_) => self.costs.header_proof,
			Request::Body(_) => self.costs.body,
			Request::Receipts(_) => self.costs.receipts,
			Request::Account(_) => self.costs.account,
			Request::Storage(_) => self.costs.storage,
			Request::Code(_) => self.costs.code,
			Request::Execution(ref req) => self.costs.transaction_proof * req.gas,
		}
	}

	/// Create initial credits.
	pub fn create_credits(&self) -> Credits {
		Credits {
			estimate: self.limit,
			recharge_point: SteadyTime::now(),
		}
	}

	/// Recharge the given credits based on time passed since last
	/// update.
	pub fn recharge(&self, credits: &mut Credits) {
		let now = SteadyTime::now();

		// recompute and update only in terms of full seconds elapsed
		// in order to keep the estimate as an underestimate.
		let elapsed = (now - credits.recharge_point).num_seconds();
		credits.recharge_point = credits.recharge_point + Duration::seconds(elapsed);

		let elapsed: U256 = elapsed.into();

		credits.estimate = ::std::cmp::min(self.limit, credits.estimate + (elapsed * self.recharge));
	}

	/// Refund some credits which were previously deducted.
	/// Does not update the recharge timestamp.
	pub fn refund(&self, credits: &mut Credits, refund_amount: U256) {
		credits.estimate = credits.estimate + refund_amount;

		if credits.estimate > self.limit {
			credits.estimate = self.limit
		}
	}
}

impl Default for FlowParams {
	fn default() -> Self {
		FlowParams {
			limit: 50_000_000.into(),
			costs: CostTable::default(),
			recharge: 100_000.into(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_serialize_cost_table() {
		let costs = CostTable::default();
		let serialized = ::rlp::encode(&costs);

		let new_costs: CostTable = ::rlp::decode(&*serialized);

		assert_eq!(costs, new_costs);
	}

	#[test]
	fn credits_mechanism() {
		use std::thread;
		use std::time::Duration;

		let flow_params = FlowParams::new(100.into(), Default::default(), 20.into());
		let mut credits =  flow_params.create_credits();

		assert!(credits.deduct_cost(101.into()).is_err());
		assert!(credits.deduct_cost(10.into()).is_ok());

		thread::sleep(Duration::from_secs(1));

		flow_params.recharge(&mut credits);

		assert_eq!(credits.estimate, 100.into());
	}
}
