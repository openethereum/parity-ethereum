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

//! LES request types.

use util::bigint::prelude::*;
use rlp::*;

/// An LES request. This defines its data format, and the format of its response type.
pub trait Request: Sized {
	/// The response type of this request.
	type Response: Response;

	/// The error type when decoding a response.
	type Error;

	/// Whether this request is empty.
	fn is_empty(&self) -> bool;

	/// The remainder of this request unfulfilled by the response. Required to return
	/// an equivalent request when provided with an empty response.
	fn remainder(&self, res: &Self::Response) -> Self;

	/// Attempt to parse raw data into a response object
	/// or an error. Behavior undefined if the raw data didn't originate from
	/// this request.
	fn parse_response(&self, raw: &[u8]) -> Result<Self::Response, Self::Error>;
}

/// Request responses. These must have a combination operation used to fill the gaps
/// in one response with data from another.
pub trait Response: Sized {
	/// Combine the two responses into one. This can only be relied on to behave correctly
	/// if `other` is a response to a sub-request of the request this response was
	/// produced from.
	fn combine(&mut self, other: Self);
}

/// A request for block bodies.
pub struct BlockBodies {
	hashes: Vec<H256>,
}

/// A response for block bodies.
pub struct BlockBodiesResponse {
	bodies: Vec<(H256, Vec<u8>)>,
}

impl Request for BlockBodies {
	type Response = BlockBodiesResponse;
	type Error = ::rlp::DecoderError;

	fn is_empty(&self) -> bool { self.hashes.is_empty() }

	fn remainder(&self, res: &Self::Response) -> Self {
		let mut remaining = Vec::new();

		let bodies = res.bodies.iter().map(|&(_, ref b) b).chain(::std::iter::repeat(&Vec::new()));
		for (hash, body) in self.hashes.iter().zip(bodies) {
			if body.is_empty() {
				remaining.push(hash);
			}
		}

		BlockBodies {
			hashes: remaining,
		}
	}

	fn parse_response(&self, raw: &[u8]) -> Result<Self::Response, Self::Error> {
		use ethcore::transaction::SignedTransaction;
		use ethcore::header::Header;

		let rlp = UntrustedRlp::new(raw);

		let mut bodies = Vec::with_capacity(self.hashes.len());

		let items = rlp.iter();
		for hash in self.hashes.iter().cloned() {
			let res_bytes = match items.next() {
				Some(rlp) => {
					// perform basic block verification.
					// TODO: custom error type?
					try!(rlp.val_at::<Vec<SignedTransaction>>(0)
						.and_then(|_| rlp.val_at::<Vec<Header>>(1)));

					try!(rlp.data()).to_owned()
				}
				None => Vec::new(),
			};

			bodies.push((hash, res_bytes));
		}

		Ok(BlockBodiesResponse {
			bodies: bodies,
		})
	}
}

impl Response for BlockBodiesResponse {
	fn identity() -> Self {
		BlockBodiesResponse {
			bodies: Vec::new(),
		}
	}

	fn combine(&mut self, other: Self) {
		let other_iter = other.bodies.into_iter();

		'a:
		for &mut (ref my_hash, ref mut my_body) in self.bodies.iter_mut() {
			loop {
				match other_iter.next() {
					Some((hash, body)) if hash == my_hash && !body.is_empty() => {
						*my_body = body.to_owned();
						break
					}
					Some(_) => continue,
					None => break 'a,
				}
			}
		}
	}
}