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

use request::{self, Request};

/// Manages buffer flow costs for specific requests.
pub struct FlowManager;

impl FlowManager {
	/// Estimate the maximum cost of this request.
	pub fn estimate_cost(&self, req: &request::Request) -> usize {
		unimplemented!()
	}

	/// Get an exact cost based on request kind and amount of requests fulfilled.
	pub fn exact_cost(&self, kind: request::Kind, amount: usize) -> usize {
		unimplemented!()
	}
}

