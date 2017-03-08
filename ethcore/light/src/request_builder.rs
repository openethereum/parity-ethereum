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
	OutputKind, Output, NoSuchOutput, Response,
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
	/// references a non-existant output of a prior request.
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

	/// Convert this into a "requests" object.
	pub fn build(self) -> Requests {
		Requests {
			output_kinds: self.output_kinds,
			outputs: HashMap::new(),
			requests: self.requests,
			offset: 0,
		}
	}
}

/// Requests pending responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requests {
	output_kinds: HashMap<(usize, usize), OutputKind>,
	outputs: HashMap<(usize, usize), Output>,
	requests: Vec<Request>,
	offset: usize, // offset for splitting.
}

impl Requests {
	/// For each request, produce responses for each.
	/// The responses vector produced goes up to the point where the responder
	/// first returns `None`, an invalid response, or until all requests have been responded to.
	pub fn respond_to_all<F>(self, responder: F) -> Vec<Response>
		where F: Fn(CompleteRequest) -> Option<Response>
	{
		let mut responses = Vec::new();
		let mut found_bad = false;
		let offset = self.offset;
		let output_kinds = self.output_kinds;
		let mut outputs = self.outputs;
		for (idx, req) in self.requests.into_iter().enumerate().map(|(idx, req)| (idx + offset, req)) {
			let complete = req.fill(|req_idx, out_idx| outputs.get(&(req_idx, out_idx)).cloned().ok_or(NoSuchOutput))
				.expect("All outputs checked as invariant of `Requests` object; qed");

			match responder(complete) {
				Some(response) => {
					response.fill_outputs(|out_idx, output| {
						match output_kinds.get(&(idx, out_idx)) {
							None => {},
							Some(out) => if out == &output.kind() {
								outputs.insert((idx, out_idx), output);
							} else {
								// output kind doesn't match expected.
								found_bad = true;
							}
						}
					});

					if found_bad {
						return responses;
					}

					responses.push(response);
				}
				None => return responses,
			}
		}

		responses
	}

	/// Get access to the underlying slice of requests.
	pub fn requests(&self) -> &[Request] { &self.requests }
}
