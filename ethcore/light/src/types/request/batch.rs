// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Request chain builder utility.
//! Push requests with `push`. Back-references and data required to verify responses must be
//! supplied as well.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use request::{
	IncompleteRequest, OutputKind, Output, NoSuchOutput, ResponseError, ResponseLike,
};

/// Build chained requests. Push them onto the series with `push`,
/// and produce a `Batch` object with `build`. Outputs are checked for consistency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Builder<T> {
	output_kinds: HashMap<(usize, usize), OutputKind>,
	requests: Vec<T>,
}

impl<T> Default for Builder<T> {
	fn default() -> Self {
		Builder {
			output_kinds: HashMap::new(),
			requests: Vec::new(),
		}
	}
}

impl<T: IncompleteRequest> Builder<T> {
	/// Attempt to push a request onto the request chain. Fails if the request
	/// references a non-existent output of a prior request.
	pub fn push(&mut self, request: T) -> Result<(), NoSuchOutput> {
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

	/// Convert this into a "batch" object.
	pub fn build(self) -> Batch<T> {
		Batch {
			outputs: HashMap::new(),
			requests: self.requests,
			answered: 0,
		}
	}
}

/// Requests pending responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch<T> {
	outputs: HashMap<(usize, usize), Output>,
	requests: Vec<T>,
	answered: usize,
}

impl<T> Batch<T> {
	/// Get access to the underlying slice of requests.
	// TODO: unimplemented -> Vec<Request>, // do we _have to_ allocate?
	pub fn requests(&self) -> &[T] { &self.requests }

	/// Get the number of answered requests.
	pub fn num_answered(&self) -> usize { self.answered }

	/// Whether the batch is complete.
	pub fn is_complete(&self) -> bool {
		self.answered == self.requests.len()
	}

	/// Map requests from one type into another.
	pub fn map_requests<F, U>(self, f: F) -> Batch<U>
		where F: FnMut(T) -> U, U: IncompleteRequest
	{
		Batch {
			outputs: self.outputs,
			requests: self.requests.into_iter().map(f).collect(),
			answered: self.answered,
		}
	}
}

impl<T: IncompleteRequest + Clone> Batch<T> {
	/// Get the next request as a filled request. Returns `None` when all requests answered.
	pub fn next_complete(&self) -> Option<T::Complete> {
		if self.is_complete() {
			None
		} else {
			Some(self.requests[self.answered].clone()
				.complete()
				.expect("All outputs checked as invariant of `Batch` object; qed"))
		}
	}

	/// Sweep through all unanswered requests, filling them as necessary.
	pub fn fill_unanswered(&mut self) {
		let outputs = &mut self.outputs;

		for req in self.requests.iter_mut().skip(self.answered) {
			req.fill(|req_idx, out_idx| outputs.get(&(req_idx, out_idx)).cloned().ok_or(NoSuchOutput))
		}
	}

	/// Supply a response, asserting its correctness.
	/// Fill outputs based upon it.
	pub fn supply_response_unchecked<R: ResponseLike>(&mut self, response: &R) {
		if self.is_complete() { return }

		let outputs = &mut self.outputs;
		let idx = self.answered;
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
	}
}

impl<T: super::CheckedRequest + Clone> Batch<T> {
	/// Supply a response for the next request.
	/// Fails on: wrong request kind, all requests answered already.
	pub fn supply_response(&mut self, env: &T::Environment, response: &T::Response)
		-> Result<T::Extract, ResponseError<T::Error>>
	{
		let idx = self.answered;

		// check validity.
		if idx == self.requests.len() { return Err(ResponseError::Unexpected) }
		let completed = self.next_complete()
			.expect("only fails when all requests have been answered; this just checked against; qed");

		let extracted = self.requests[idx]
			.check_response(&completed, env, response).map_err(ResponseError::Validity)?;

		self.supply_response_unchecked(response);
		Ok(extracted)
	}
}

impl Batch<super::Request> {
	/// For each request, produce a response.
	/// The responses vector produced goes up to the point where the responder
	/// first returns `None`, an invalid response, or until all requests have been responded to.
	pub fn respond_to_all<F>(mut self, responder: F) -> Vec<super::Response>
		where F: Fn(super::CompleteRequest) -> Option<super::Response>
	{
		let mut responses = Vec::new();

		while let Some(response) = self.next_complete().and_then(&responder) {
			match self.supply_response(&(), &response) {
				Ok(()) => responses.push(response),
				Err(e) => {
					debug!(target: "pip", "produced bad response to request: {:?}", e);
					return responses;
				}
			}
		}

		responses
	}
}

impl<T: IncompleteRequest> Deref for Batch<T> {
	type Target = [T];

	fn deref(&self) -> &[T] {
		&self.requests[..]
	}
}

impl<T: IncompleteRequest> DerefMut for Batch<T> {
	fn deref_mut(&mut self) -> &mut [T] {
		&mut self.requests[..]
	}
}

#[cfg(test)]
mod tests {
	use request::*;
	use super::Builder;
	use ethereum_types::H256;

	#[test]
	fn all_scalar() {
		let mut builder = Builder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(),
		})).unwrap();
		builder.push(Request::Receipts(IncompleteReceiptsRequest {
			hash: H256::zero().into(),
		})).unwrap();
	}

	#[test]
	#[should_panic]
	fn missing_backref() {
		let mut builder = Builder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: Field::BackReference(100, 3),
		})).unwrap();
	}

	#[test]
	#[should_panic]
	fn wrong_kind() {
		let mut builder = Builder::default();
		assert!(builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(),
		})).is_ok());
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: Field::BackReference(0, 0),
		})).unwrap();
	}

	#[test]
	fn good_backreference() {
		let mut builder = Builder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(), // header proof puts hash at output 0.
		})).unwrap();
		builder.push(Request::Receipts(IncompleteReceiptsRequest {
			hash: Field::BackReference(0, 0),
		})).unwrap();
	}

	#[test]
	fn batch_tx_index_backreference() {
		let mut builder = Builder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(), // header proof puts hash at output 0.
		})).unwrap();
		builder.push(Request::TransactionIndex(IncompleteTransactionIndexRequest {
			hash: Field::BackReference(0, 0),
		})).unwrap();

		let mut batch = builder.build();
		batch.requests[1].fill(|_req_idx, _out_idx| Ok(Output::Hash(H256::from_low_u64_be(42))));

		assert!(batch.next_complete().is_some());
		batch.answered += 1;
		assert!(batch.next_complete().is_some());
	}

	#[test]
	fn batch_tx_index_backreference_public_api() {
		let mut builder = Builder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(), // header proof puts hash at output 0.
		})).unwrap();
		builder.push(Request::TransactionIndex(IncompleteTransactionIndexRequest {
			hash: Field::BackReference(0, 0),
		})).unwrap();

		let mut batch = builder.build();

		assert!(batch.next_complete().is_some());
		let hdr_proof_res = header_proof::Response {
			proof: vec![],
			hash: H256::from_low_u64_be(12),
			td: 21.into(),
		};
		batch.supply_response_unchecked(&hdr_proof_res);

		assert!(batch.next_complete().is_some());
	}

	#[test]
	fn batch_receipts_backreference() {
		let mut builder = Builder::default();
		builder.push(Request::HeaderProof(IncompleteHeaderProofRequest {
			num: 100.into(), // header proof puts hash at output 0.
		})).unwrap();
		builder.push(Request::Receipts(IncompleteReceiptsRequest {
			hash: Field::BackReference(0, 0),
		})).unwrap();

		let mut batch = builder.build();
		batch.requests[1].fill(|_req_idx, _out_idx| Ok(Output::Hash(H256::from_low_u64_be(42))));

		assert!(batch.next_complete().is_some());
		batch.answered += 1;
		assert!(batch.next_complete().is_some());
	}
}
