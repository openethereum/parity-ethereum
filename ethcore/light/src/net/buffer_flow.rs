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

//! LES buffer flow management.
//!
//! Every request in the LES protocol leads to a reduction
//! of the requester's buffer value as a rate-limiting mechanism.
//! This buffer value will recharge at a set rate.
//!
//! This module provides an interface for configuration of buffer
//! flow costs and recharge rates.

use request;
use super::packet;
use super::error::Error;

use rlp::*;
use util::U256;
use time::{Duration, SteadyTime};

/// A request cost specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost(pub U256, pub U256);

/// Buffer value.
///
/// Produced and recharged using `FlowParams`.
/// Definitive updates can be made as well -- these will reset the recharge
/// point to the time of the update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
	estimate: U256,
	recharge_point: SteadyTime,
}

impl Buffer {
	/// Get the current buffer value.
	pub fn current(&self) -> U256 { self.estimate.clone() }

	/// Make a definitive update.
	/// This will be the value obtained after receiving
	/// a response to a request.
	pub fn update_to(&mut self, value: U256) {
		self.estimate = value;
		self.recharge_point = SteadyTime::now();
	}

	/// Attempt to apply the given cost to the buffer.
	///
	/// If successful, the cost will be deducted successfully.
	///
	/// If unsuccessful, the structure will be unaltered an an
	/// error will be produced.
	pub fn deduct_cost(&mut self, cost: U256) -> Result<(), Error> {
		match cost > self.estimate {
			true => Err(Error::BufferEmpty),
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
	headers: Cost,
	bodies: Cost,
	receipts: Cost,
	state_proofs: Cost,
	contract_codes: Cost,
	header_proofs: Cost,
}

impl Default for CostTable {
	fn default() -> Self {
		// arbitrarily chosen constants.
		CostTable {
			headers: Cost(100000.into(), 10000.into()),
			bodies: Cost(150000.into(), 15000.into()),
			receipts: Cost(50000.into(), 5000.into()),
			state_proofs: Cost(250000.into(), 25000.into()),
			contract_codes: Cost(200000.into(), 20000.into()),
			header_proofs: Cost(150000.into(), 15000.into()),
		}
	}
}

impl RlpEncodable for CostTable {
	fn rlp_append(&self, s: &mut RlpStream) {
		fn append_cost(s: &mut RlpStream, msg_id: u8, cost: &Cost) {
			s.begin_list(3)
				.append(&msg_id)
				.append(&cost.0)
				.append(&cost.1);
		}

		s.begin_list(6);

		append_cost(s, packet::GET_BLOCK_HEADERS, &self.headers);
		append_cost(s, packet::GET_BLOCK_BODIES, &self.bodies);
		append_cost(s, packet::GET_RECEIPTS, &self.receipts);
		append_cost(s, packet::GET_PROOFS, &self.state_proofs);
		append_cost(s, packet::GET_CONTRACT_CODES, &self.contract_codes);
		append_cost(s, packet::GET_HEADER_PROOFS, &self.header_proofs);
	}
}

impl RlpDecodable for CostTable {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let rlp = decoder.as_rlp();

		let mut headers = None;
		let mut bodies = None;
		let mut receipts = None;
		let mut state_proofs = None;
		let mut contract_codes = None;
		let mut header_proofs = None;

		for row in rlp.iter() {
			let msg_id: u8 = try!(row.val_at(0));
			let cost = {
				let base = try!(row.val_at(1));
				let per = try!(row.val_at(2));

				Cost(base, per)
			};

			match msg_id {
				packet::GET_BLOCK_HEADERS => headers = Some(cost),
				packet::GET_BLOCK_BODIES => bodies = Some(cost),
				packet::GET_RECEIPTS => receipts = Some(cost),
				packet::GET_PROOFS => state_proofs = Some(cost),
				packet::GET_CONTRACT_CODES => contract_codes = Some(cost),
				packet::GET_HEADER_PROOFS => header_proofs = Some(cost),
				_ => return Err(DecoderError::Custom("Unrecognized message in cost table")),
			}
		}

		Ok(CostTable {
			headers: try!(headers.ok_or(DecoderError::Custom("No headers cost specified"))),
			bodies: try!(bodies.ok_or(DecoderError::Custom("No bodies cost specified"))),
			receipts: try!(receipts.ok_or(DecoderError::Custom("No receipts cost specified"))),
			state_proofs: try!(state_proofs.ok_or(DecoderError::Custom("No proofs cost specified"))),
			contract_codes: try!(contract_codes.ok_or(DecoderError::Custom("No contract codes specified"))),
			header_proofs: try!(header_proofs.ok_or(DecoderError::Custom("No header proofs cost specified"))),
		})
	}
}

/// A buffer-flow manager handles costs, recharge, limits
#[derive(Debug, Clone, PartialEq)]
pub struct FlowParams {
	costs: CostTable,
	limit: U256,
	recharge: U256,
}

impl FlowParams {
	/// Create new flow parameters from a request cost table,
	/// buffer limit, and (minimum) rate of recharge.
	pub fn new(limit: U256, costs: CostTable, recharge: U256) -> Self {
		FlowParams {
			costs: costs,
			limit: limit,
			recharge: recharge,
		}
	}

	/// Get a reference to the buffer limit.
	pub fn limit(&self) -> &U256 { &self.limit }

	/// Get a reference to the cost table.
	pub fn cost_table(&self) -> &CostTable { &self.costs }

	/// Get a reference to the recharge rate.
	pub fn recharge_rate(&self) -> &U256 { &self.recharge }

	/// Compute the actual cost of a request, given the kind of request
	/// and number of requests made.
	pub fn compute_cost(&self, kind: request::Kind, amount: usize) -> U256 {
		let cost = match kind {
			request::Kind::Headers => &self.costs.headers,
			request::Kind::Bodies => &self.costs.bodies,
			request::Kind::Receipts => &self.costs.receipts,
			request::Kind::StateProofs => &self.costs.state_proofs,
			request::Kind::Codes => &self.costs.contract_codes,
			request::Kind::HeaderProofs => &self.costs.header_proofs,
		};

		let amount: U256 = amount.into();
		cost.0 + (amount * cost.1)
	}

	/// Create initial buffer parameter.
	pub fn create_buffer(&self) -> Buffer {
		Buffer {
			estimate: self.limit,
			recharge_point: SteadyTime::now(),
		}
	}

	/// Recharge the buffer based on time passed since last
	/// update.
	pub fn recharge(&self, buf: &mut Buffer) {
		let now = SteadyTime::now();

		// recompute and update only in terms of full seconds elapsed
		// in order to keep the estimate as an underestimate.
		let elapsed = (now - buf.recharge_point).num_seconds();
		buf.recharge_point = buf.recharge_point + Duration::seconds(elapsed);

		let elapsed: U256 = elapsed.into();

		buf.estimate = ::std::cmp::min(self.limit, buf.estimate + (elapsed * self.recharge));
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
	fn buffer_mechanism() {
		use std::thread;
		use std::time::Duration;

		let flow_params = FlowParams::new(100.into(), Default::default(), 20.into());
		let mut buffer =  flow_params.create_buffer();

		assert!(buffer.deduct_cost(101.into()).is_err());
		assert!(buffer.deduct_cost(10.into()).is_ok());

		thread::sleep(Duration::from_secs(1));

		flow_params.recharge(&mut buffer);

		assert_eq!(buffer.estimate, 100.into());
	}
}