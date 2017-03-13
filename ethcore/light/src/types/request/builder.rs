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

//! Request chain builder utility.
//! Push requests with `push`. Back-references and data required to verify responses must be
//! supplied as well.

use std::collections::HashMap;
use request::{
	IncompleteRequest, CompleteRequest, Request,
	OutputKind, Output, NoSuchOutput, Response, ResponseError,
};

/// Build chained requests. Push them onto the series with `push`,
/// and produce a `Requests` object with `build`. Outputs are checked for consistency.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RequestBuilder {
	output_kinds: HashMap<(usize, usize), OutputKind>,
	requests: Vec<Request>,
}

impl RequestBuilder {
	/// Attempt to push a request onto the request chain. Fails if the request
	/// references a non-existent output of a prior request.
	pub fn push(&mut self, request: Request) -> Result<(), NoSuchOutput> {
		request.check_outputs(|req, idx, kind| {
			match self.output_kinds.get(&(req, idx)) {
				Some(k) if k == &kind => Ok(()),
				_ => Err(NoSuchOutput),
			}
		})?;
		let req_idx = self.requests.len();
		request.note_outputs(|idx, kind| { self.output_kinds.insert((req_idx, idx), kind); });
		self.requests.push(request);
		Ok(())
	}

	/// Get a reference to the output kinds map.
	pub fn output_kinds(&self) -> &HashMap<(usize, usize), OutputKind> {
		&self.output_kinds
	}

	/// Convert this into a "requests" object.
	pub fn build(self) -> Requests {
		Requests {
			outputs: HashMap::new(),
			requests: self.requests,
			answered: 0,
		}
	}
}

/// Requests pending responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requests {
	outputs: HashMap<(usize, usize), Output>,
	requests: Vec<Request>,
	answered: usize,
}

impl Requests {
	/// For each request, produce responses for each.
	/// The responses vector produced goes up to the point where the responder
	/// first returns `None`, an invalid response, or until all requests have been responded to.
	pub fn respond_to_all<F>(mut self, responder: F) -> Vec<Response>
		where F: Fn(CompleteRequest) -> Option<Response>
	{
		let mut responses = Vec::new();

		while let Some(response) = self.next_complete().and_then(&responder) {
			match self.supply_response(&response) {
				Ok(()) => responses.push(response),
				Err(e) => {
					debug!(target: "pip", "produced bad response to request: {:?}", e);
					return responses;
				}
			}
		}

		responses
	}

	/// Get access to the underlying slice of requests.
	// TODO: unimplemented -> Vec<Request>, // do we _have to_ allocate?
	pub fn requests(&self) -> &[Request] { &self.requests }

	/// Get the number of answered requests.
	pub fn num_answered(&self) -> usize { self.answered }

	/// Get the next request as a filled request. Returns `None` when all requests answered.
	pub fn next_complete(&self) -> Option<CompleteRequest> {
		if self.answered == self.requests.len() {
			None
		} else {
			Some(self.requests[self.answered].clone()
				.complete()
				.expect("All outputs checked as invariant of `Requests` object; qed"))
		}
	}

	/// Supply a response for the next request.
	/// Fails on: wrong request kind, all requests answered already.
	pub fn supply_response(&mut self, response: &Response) -> Result<(), ResponseError> {
		let idx = self.answered;

		// check validity.
		if idx == self.requests.len() { return Err(ResponseError::Unexpected) }
		if self.requests[idx].kind() != response.kind() { return Err(ResponseError::WrongKind) }

		let outputs = &mut self.outputs;
		response.fill_outputs(|out_idx, output| {
			// we don't need to check output kinds here because all back-references
			// are validated in the builder.
			// TODO: optimization for only storing outputs we "care about"?
			outputs.insert((idx, out_idx), output);
		});

		self.answered += 1;

		// fill as much of the next request as we can.
		if let Some(ref mut req) = self.requests.get_mut(self.answered) {
			req.fill(|req_idx, out_idx| outputs.get(&(req_idx, out_idx)).cloned().ok_or(NoSuchOutput))
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use request::*;
	use super::RequestBuilder;
	use util::H256;

	#[test]
	fn all_scalar() {
		let mut builder = RequestBuilder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(),
		})).unwrap();
		builder.push(Request::Receipts(IncompleteReceiptsRequest {
			hash: H256::default().into(),
		})).unwrap();
	}

	#[test]
	#[should_panic]
	fn missing_backref() {
		let mut builder = RequestBuilder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: Field::BackReference(100, 3),
		})).unwrap();
	}

	#[test]
	#[should_panic]
	fn wrong_kind() {
		let mut builder = RequestBuilder::default();
		assert!(builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(),
		})).is_ok());
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: Field::BackReference(0, 0),
		})).unwrap();
	}

	#[test]
	fn good_backreference() {
		let mut builder = RequestBuilder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(), // header proof puts hash at output 0.
		})).unwrap();
		builder.push(Request::Receipts(IncompleteReceiptsRequest {
			hash: Field::BackReference(0, 0),
		})).unwrap();
	}
}
