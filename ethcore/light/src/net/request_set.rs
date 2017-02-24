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

//! Pending request set.
//!
//! Stores pending requests and does timeout computation according to the rule
//! that only the earliest submitted request within the structure may time out.
//!
//! Whenever a request becomes the earliest, its timeout period begins at that moment.

use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;

use request::{self, Request};
use net::{timeout, ReqId};

use time::{Duration, SteadyTime};

/// Request set.
#[derive(Debug)]
pub struct RequestSet {
	counter: u64,
	base: Option<SteadyTime>,
	ids: HashMap<ReqId, u64>,
	reqs: BTreeMap<u64, Request>,
}

impl Default for RequestSet {
	fn default() -> Self {
		RequestSet {
			counter: 0,
			base: None,
			ids: HashMap::new(),
			reqs: BTreeMap::new(),
		}
	}
}

impl RequestSet {
	/// Push a request onto the stack.
	pub fn insert(&mut self, req_id: ReqId, req: Request, now: SteadyTime) {
		let counter = self.counter;
		self.ids.insert(req_id, counter);
		self.reqs.insert(counter, req);

		if self.reqs.keys().next().map_or(true, |x| *x == counter) {
			self.base = Some(now);
		}

		self.counter += 1;
	}

	/// Remove a request from the stack.
	pub fn remove(&mut self, req_id: &ReqId, now: SteadyTime) -> Option<Request> {
		let id = match self.ids.remove(&req_id) {
			Some(id) => id,
			None => return None,
		};

		let req = self.reqs.remove(&id).expect("entry in `ids` implies entry in `reqs`; qed");

		match self.reqs.keys().next() {
			Some(k) if *k > id => self.base = Some(now),
			None => self.base = None,
			_ => {}
		}

		Some(req)
	}

	/// Check for timeout against the given time. Returns true if
	/// has timed out, false otherwise.
	pub fn check_timeout(&self, now: SteadyTime) -> bool {
		let base = match self.base.as_ref().cloned() {
			Some(base) => base,
			None => return false,
		};

		let kind = self.reqs.values()
			.next()
			.map(|r| r.kind())
			.expect("base time implies `reqs` non-empty; qed");

		let kind_timeout = match kind {
			request::Kind::Headers => timeout::HEADERS,
			request::Kind::Bodies => timeout::BODIES,
			request::Kind::Receipts => timeout::RECEIPTS,
			request::Kind::StateProofs => timeout::PROOFS,
			request::Kind::Codes => timeout::CONTRACT_CODES,
			request::Kind::HeaderProofs => timeout::HEADER_PROOFS,
		};

		base + Duration::milliseconds(kind_timeout) <= now
	}

	/// Collect all pending request ids.
	pub fn collect_ids<F>(&self) -> F where F: FromIterator<ReqId> {
		self.ids.keys().cloned().collect()
	}

	/// Number of requests in the set.
	pub fn len(&self) -> usize {
		self.ids.len()
	}

	/// Whether the set is empty.
	pub fn is_empty(&self) -> bool { self.len() == 0 }
}

#[cfg(test)]
mod tests {
	use net::{timeout, ReqId};
	use request::{Request, Receipts};
	use time::{SteadyTime, Duration};
	use super::RequestSet;

	#[test]
	fn multi_timeout() {
		let test_begin = SteadyTime::now();
		let mut req_set = RequestSet::default();

		let the_req = Request::Receipts(Receipts { block_hashes: Vec::new() });
		req_set.insert(ReqId(0), the_req.clone(), test_begin);
		req_set.insert(ReqId(1), the_req, test_begin + Duration::seconds(1));

		assert_eq!(req_set.base, Some(test_begin));

		let test_end = test_begin + Duration::milliseconds(timeout::RECEIPTS);
		assert!(req_set.check_timeout(test_end));

		req_set.remove(&ReqId(0), test_begin + Duration::seconds(1)).unwrap();
		assert!(!req_set.check_timeout(test_end));
		assert!(req_set.check_timeout(test_end + Duration::seconds(1)));
	}
}
