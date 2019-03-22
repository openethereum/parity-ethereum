// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Light protocol request types.

use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};
use ethereum_types::H256;

mod batch;

// re-exports of request types.
pub use self::header::{
	Complete as CompleteHeadersRequest,
	Incomplete as IncompleteHeadersRequest,
	Response as HeadersResponse
};
pub use self::header_proof::{
	Complete as CompleteHeaderProofRequest,
	Incomplete as IncompleteHeaderProofRequest,
	Response as HeaderProofResponse
};
pub use self::transaction_index::{
	Complete as CompleteTransactionIndexRequest,
	Incomplete as IncompleteTransactionIndexRequest,
	Response as TransactionIndexResponse
};
pub use self::block_body::{
	Complete as CompleteBodyRequest,
	Incomplete as IncompleteBodyRequest,
	Response as BodyResponse
};
pub use self::block_receipts::{
	Complete as CompleteReceiptsRequest,
	Incomplete as IncompleteReceiptsRequest,
	Response as ReceiptsResponse
};
pub use self::account::{
	Complete as CompleteAccountRequest,
	Incomplete as IncompleteAccountRequest,
	Response as AccountResponse,
};
pub use self::storage::{
	Complete as CompleteStorageRequest,
	Incomplete as IncompleteStorageRequest,
	Response as StorageResponse
};
pub use self::contract_code::{
	Complete as CompleteCodeRequest,
	Incomplete as IncompleteCodeRequest,
	Response as CodeResponse,
};
pub use self::execution::{
	Complete as CompleteExecutionRequest,
	Incomplete as IncompleteExecutionRequest,
	Response as ExecutionResponse,
};
pub use self::epoch_signal::{
	Complete as CompleteSignalRequest,
	Incomplete as IncompleteSignalRequest,
	Response as SignalResponse,
};

pub use self::batch::{Batch, Builder};

/// Error indicating a reference to a non-existent or wrongly-typed output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoSuchOutput;

/// Wrong kind of response corresponding to request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrongKind;

/// Error on processing a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseError<T> {
	/// Error in validity.
	Validity(T),
	/// No responses expected.
	Unexpected,
}

/// An input to a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field<T> {
	/// A pre-specified input.
	Scalar(T),
	/// An input which can be resolved later on.
	/// (Request index, output index)
	BackReference(usize, usize),
}

impl<T> Field<T> {
	/// Helper for creating a new back-reference field.
	pub fn back_ref(idx: usize, req: usize) -> Self {
		Field::BackReference(idx, req)
	}

	/// map a scalar into some other item.
	pub fn map<F, U>(self, f: F) -> Field<U> where F: FnOnce(T) -> U {
		match self {
			Field::Scalar(x) => Field::Scalar(f(x)),
			Field::BackReference(req, idx) => Field::BackReference(req, idx),
		}
	}

	/// Attempt to get a reference to the inner scalar.
	pub fn as_ref(&self) -> Option<&T> {
		match *self {
			Field::Scalar(ref x) => Some(x),
			Field::BackReference(_, _) => None,
		}
	}

	// attempt conversion into scalar value.
	fn into_scalar(self) -> Result<T, NoSuchOutput> {
		match self {
			Field::Scalar(val) => Ok(val),
			_ => Err(NoSuchOutput),
		}
	}

	fn adjust_req<F>(&mut self, mut mapping: F) where F: FnMut(usize) -> usize {
		if let Field::BackReference(ref mut req_idx, _) = *self {
			*req_idx = mapping(*req_idx)
		}
	}
}

impl<T> From<T> for Field<T> {
	fn from(val: T) -> Self {
		Field::Scalar(val)
	}
}

impl<T: Decodable> Decodable for Field<T> {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		match rlp.val_at::<u8>(0)? {
			0 => Ok(Field::Scalar(rlp.val_at::<T>(1)?)),
			1 => Ok({
				let inner_rlp = rlp.at(1)?;
				Field::BackReference(inner_rlp.val_at(0)?, inner_rlp.val_at(1)?)
			}),
			_ => Err(DecoderError::Custom("Unknown discriminant for PIP field.")),
		}
	}
}

impl<T: Encodable> Encodable for Field<T> {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		match *self {
			Field::Scalar(ref data) => {
				s.append(&0u8).append(data);
			}
			Field::BackReference(ref req, ref idx) => {
				s.append(&1u8).begin_list(2).append(req).append(idx);
			}
		}
	}
}

/// Request outputs which can be reused as inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
	/// A 32-byte hash output.
	Hash(H256),
	/// An unsigned-integer output.
	Number(u64),
}

impl Output {
	/// Get the output kind.
	pub fn kind(&self) -> OutputKind {
		match *self {
			Output::Hash(_) => OutputKind::Hash,
			Output::Number(_) => OutputKind::Number,
		}
	}
}

/// Response output kinds which can be used as back-references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputKind {
	/// A 32-byte hash output.
	Hash,
	/// An unsigned-integer output.
	Number,
}

/// Either a hash or a number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashOrNumber {
	/// Block hash variant.
	Hash(H256),
	/// Block number variant.
	Number(u64),
}

impl From<H256> for HashOrNumber {
	fn from(hash: H256) -> Self {
		HashOrNumber::Hash(hash)
	}
}

impl From<u64> for HashOrNumber {
	fn from(num: u64) -> Self {
		HashOrNumber::Number(num)
	}
}

impl Decodable for HashOrNumber {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.as_val::<H256>().map(HashOrNumber::Hash)
			.or_else(|_| rlp.as_val().map(HashOrNumber::Number))
	}
}

impl Encodable for HashOrNumber {
	fn rlp_append(&self, s: &mut RlpStream) {
		match *self {
			HashOrNumber::Hash(ref hash) => s.append(hash),
			HashOrNumber::Number(ref num) => s.append(num),
		};
	}
}

/// Type alias for "network requests".
pub type NetworkRequests = Batch<Request>;

/// All request types, as they're sent over the network.
/// They may be incomplete, with back-references to outputs
/// of prior requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
	/// A request for block headers.
	Headers(IncompleteHeadersRequest),
	/// A request for a header proof (from a CHT)
	HeaderProof(IncompleteHeaderProofRequest),
	/// A request for a transaction index by hash.
	TransactionIndex(IncompleteTransactionIndexRequest),
	/// A request for a block's receipts.
	Receipts(IncompleteReceiptsRequest),
	/// A request for a block body.
	Body(IncompleteBodyRequest),
	/// A request for a merkle proof of an account.
	Account(IncompleteAccountRequest),
	/// A request for a merkle proof of contract storage.
	Storage(IncompleteStorageRequest),
	/// A request for contract code.
	Code(IncompleteCodeRequest),
	/// A request for proof of execution,
	Execution(IncompleteExecutionRequest),
	/// A request for an epoch signal.
	Signal(IncompleteSignalRequest),
}

/// All request types, in an answerable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompleteRequest {
	/// A request for block headers.
	Headers(CompleteHeadersRequest),
	/// A request for a header proof (from a CHT)
	HeaderProof(CompleteHeaderProofRequest),
	/// A request for a transaction index by hash.
	TransactionIndex(CompleteTransactionIndexRequest),
	/// A request for a block's receipts.
	Receipts(CompleteReceiptsRequest),
	/// A request for a block body.
	Body(CompleteBodyRequest),
	/// A request for a merkle proof of an account.
	Account(CompleteAccountRequest),
	/// A request for a merkle proof of contract storage.
	Storage(CompleteStorageRequest),
	/// A request for contract code.
	Code(CompleteCodeRequest),
	/// A request for proof of execution,
	Execution(CompleteExecutionRequest),
	/// A request for an epoch signal.
	Signal(CompleteSignalRequest),
}

impl CompleteRequest {
	/// Inspect the kind of this response.
	pub fn kind(&self) -> Kind {
		match *self {
			CompleteRequest::Headers(_) => Kind::Headers,
			CompleteRequest::HeaderProof(_) => Kind::HeaderProof,
			CompleteRequest::TransactionIndex(_) => Kind::TransactionIndex,
			CompleteRequest::Receipts(_) => Kind::Receipts,
			CompleteRequest::Body(_) => Kind::Body,
			CompleteRequest::Account(_) => Kind::Account,
			CompleteRequest::Storage(_) => Kind::Storage,
			CompleteRequest::Code(_) => Kind::Code,
			CompleteRequest::Execution(_) => Kind::Execution,
			CompleteRequest::Signal(_) => Kind::Signal,
		}
	}
}

impl Request {
	/// Get the request kind.
	pub fn kind(&self) -> Kind {
		match *self {
			Request::Headers(_) => Kind::Headers,
			Request::HeaderProof(_) => Kind::HeaderProof,
			Request::TransactionIndex(_) => Kind::TransactionIndex,
			Request::Receipts(_) => Kind::Receipts,
			Request::Body(_) => Kind::Body,
			Request::Account(_) => Kind::Account,
			Request::Storage(_) => Kind::Storage,
			Request::Code(_) => Kind::Code,
			Request::Execution(_) => Kind::Execution,
			Request::Signal(_) => Kind::Signal,
		}
	}
}

impl Decodable for Request {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		match rlp.val_at::<Kind>(0)? {
			Kind::Headers => Ok(Request::Headers(rlp.val_at(1)?)),
			Kind::HeaderProof => Ok(Request::HeaderProof(rlp.val_at(1)?)),
			Kind::TransactionIndex => Ok(Request::TransactionIndex(rlp.val_at(1)?)),
			Kind::Receipts => Ok(Request::Receipts(rlp.val_at(1)?)),
			Kind::Body => Ok(Request::Body(rlp.val_at(1)?)),
			Kind::Account => Ok(Request::Account(rlp.val_at(1)?)),
			Kind::Storage => Ok(Request::Storage(rlp.val_at(1)?)),
			Kind::Code => Ok(Request::Code(rlp.val_at(1)?)),
			Kind::Execution => Ok(Request::Execution(rlp.val_at(1)?)),
			Kind::Signal => Ok(Request::Signal(rlp.val_at(1)?)),
		}
	}
}

impl Encodable for Request {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);

		// hack around https://github.com/paritytech/parity-ethereum/issues/4356
		Encodable::rlp_append(&self.kind(), s);

		match *self {
			Request::Headers(ref req) => s.append(req),
			Request::HeaderProof(ref req) => s.append(req),
			Request::TransactionIndex(ref req) => s.append(req),
			Request::Receipts(ref req) => s.append(req),
			Request::Body(ref req) => s.append(req),
			Request::Account(ref req) => s.append(req),
			Request::Storage(ref req) => s.append(req),
			Request::Code(ref req) => s.append(req),
			Request::Execution(ref req) => s.append(req),
			Request::Signal(ref req) => s.append(req),
		};
	}
}

impl IncompleteRequest for Request {
	type Complete = CompleteRequest;
	type Response = Response;

	fn check_outputs<F>(&self, f: F) -> Result<(), NoSuchOutput>
		where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
	{
		match *self {
			Request::Headers(ref req) => req.check_outputs(f),
			Request::HeaderProof(ref req) => req.check_outputs(f),
			Request::TransactionIndex(ref req) => req.check_outputs(f),
			Request::Receipts(ref req) => req.check_outputs(f),
			Request::Body(ref req) => req.check_outputs(f),
			Request::Account(ref req) => req.check_outputs(f),
			Request::Storage(ref req) => req.check_outputs(f),
			Request::Code(ref req) => req.check_outputs(f),
			Request::Execution(ref req) => req.check_outputs(f),
			Request::Signal(ref req) => req.check_outputs(f),
		}
	}

	fn note_outputs<F>(&self, f: F) where F: FnMut(usize, OutputKind) {
		match *self {
			Request::Headers(ref req) => req.note_outputs(f),
			Request::HeaderProof(ref req) => req.note_outputs(f),
			Request::TransactionIndex(ref req) => req.note_outputs(f),
			Request::Receipts(ref req) => req.note_outputs(f),
			Request::Body(ref req) => req.note_outputs(f),
			Request::Account(ref req) => req.note_outputs(f),
			Request::Storage(ref req) => req.note_outputs(f),
			Request::Code(ref req) => req.note_outputs(f),
			Request::Execution(ref req) => req.note_outputs(f),
			Request::Signal(ref req) => req.note_outputs(f),
		}
	}

	fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
		match *self {
			Request::Headers(ref mut req) => req.fill(oracle),
			Request::HeaderProof(ref mut req) => req.fill(oracle),
			Request::TransactionIndex(ref mut req) => req.fill(oracle),
			Request::Receipts(ref mut req) => req.fill(oracle),
			Request::Body(ref mut req) => req.fill(oracle),
			Request::Account(ref mut req) => req.fill(oracle),
			Request::Storage(ref mut req) => req.fill(oracle),
			Request::Code(ref mut req) => req.fill(oracle),
			Request::Execution(ref mut req) => req.fill(oracle),
			Request::Signal(ref mut req) => req.fill(oracle),
		}
	}

	fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
		match self {
			Request::Headers(req) => req.complete().map(CompleteRequest::Headers),
			Request::HeaderProof(req) => req.complete().map(CompleteRequest::HeaderProof),
			Request::TransactionIndex(req) => req.complete().map(CompleteRequest::TransactionIndex),
			Request::Receipts(req) => req.complete().map(CompleteRequest::Receipts),
			Request::Body(req) => req.complete().map(CompleteRequest::Body),
			Request::Account(req) => req.complete().map(CompleteRequest::Account),
			Request::Storage(req) => req.complete().map(CompleteRequest::Storage),
			Request::Code(req) => req.complete().map(CompleteRequest::Code),
			Request::Execution(req) => req.complete().map(CompleteRequest::Execution),
			Request::Signal(req) => req.complete().map(CompleteRequest::Signal),
		}
	}

	fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
		match *self {
			Request::Headers(ref mut req) => req.adjust_refs(mapping),
			Request::HeaderProof(ref mut req) => req.adjust_refs(mapping),
			Request::TransactionIndex(ref mut req) => req.adjust_refs(mapping),
			Request::Receipts(ref mut req) => req.adjust_refs(mapping),
			Request::Body(ref mut req) => req.adjust_refs(mapping),
			Request::Account(ref mut req) => req.adjust_refs(mapping),
			Request::Storage(ref mut req) => req.adjust_refs(mapping),
			Request::Code(ref mut req) => req.adjust_refs(mapping),
			Request::Execution(ref mut req) => req.adjust_refs(mapping),
			Request::Signal(ref mut req) => req.adjust_refs(mapping),
		}
	}
}

impl CheckedRequest for Request {
	type Extract = ();
	type Error = WrongKind;
	type Environment = ();

	fn check_response(&self, _: &Self::Complete, _: &(), response: &Response) -> Result<(), WrongKind> {
		if self.kind() == response.kind() {
			Ok(())
		} else {
			Err(WrongKind)
		}
	}
}

/// Kinds of requests.
/// Doubles as the "ID" field of the request.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
	/// A request for headers.
	Headers = 0,
	/// A request for a header proof.
	HeaderProof = 1,
	/// A request for a transaction index.
	TransactionIndex = 2,
	/// A request for block receipts.
	Receipts = 3,
	/// A request for a block body.
	Body = 4,
	/// A request for an account + merkle proof.
	Account = 5,
	/// A request for contract storage + merkle proof
	Storage = 6,
	/// A request for contract.
	Code = 7,
	/// A request for transaction execution + state proof.
	Execution = 8,
	/// A request for epoch transition signal.
	Signal = 9,
}

impl Decodable for Kind {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		match rlp.as_val::<u8>()? {
			0 => Ok(Kind::Headers),
			1 => Ok(Kind::HeaderProof),
			2 => Ok(Kind::TransactionIndex),
			3 => Ok(Kind::Receipts),
			4 => Ok(Kind::Body),
			5 => Ok(Kind::Account),
			6 => Ok(Kind::Storage),
			7 => Ok(Kind::Code),
			8 => Ok(Kind::Execution),
			9 => Ok(Kind::Signal),
			_ => Err(DecoderError::Custom("Unknown PIP request ID.")),
		}
	}
}

impl Encodable for Kind {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&(*self as u8));
	}
}

/// All response types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
	/// A response for block headers.
	Headers(HeadersResponse),
	/// A response for a header proof (from a CHT)
	HeaderProof(HeaderProofResponse),
	/// A response for a transaction index.
	TransactionIndex(TransactionIndexResponse),
	/// A response for a block's receipts.
	Receipts(ReceiptsResponse),
	/// A response for a block body.
	Body(BodyResponse),
	/// A response for a merkle proof of an account.
	Account(AccountResponse),
	/// A response for a merkle proof of contract storage.
	Storage(StorageResponse),
	/// A response for contract code.
	Code(CodeResponse),
	/// A response for proof of execution,
	Execution(ExecutionResponse),
	/// A response for epoch change signal.
	Signal(SignalResponse),
}

impl ResponseLike for Response {
	/// Fill reusable outputs by writing them into the function.
	fn fill_outputs<F>(&self, f: F) where F: FnMut(usize, Output) {
		match *self {
			Response::Headers(ref res) => res.fill_outputs(f),
			Response::HeaderProof(ref res) => res.fill_outputs(f),
			Response::TransactionIndex(ref res) => res.fill_outputs(f),
			Response::Receipts(ref res) => res.fill_outputs(f),
			Response::Body(ref res) => res.fill_outputs(f),
			Response::Account(ref res) => res.fill_outputs(f),
			Response::Storage(ref res) => res.fill_outputs(f),
			Response::Code(ref res) => res.fill_outputs(f),
			Response::Execution(ref res) => res.fill_outputs(f),
			Response::Signal(ref res) => res.fill_outputs(f),
		}
	}
}

impl Response {
	/// Inspect the kind of this response.
	pub fn kind(&self) -> Kind {
		match *self {
			Response::Headers(_) => Kind::Headers,
			Response::HeaderProof(_) => Kind::HeaderProof,
			Response::TransactionIndex(_) => Kind::TransactionIndex,
			Response::Receipts(_) => Kind::Receipts,
			Response::Body(_) => Kind::Body,
			Response::Account(_) => Kind::Account,
			Response::Storage(_) => Kind::Storage,
			Response::Code(_) => Kind::Code,
			Response::Execution(_) => Kind::Execution,
			Response::Signal(_) => Kind::Signal,
		}
	}
}

impl Decodable for Response {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		match rlp.val_at::<Kind>(0)? {
			Kind::Headers => Ok(Response::Headers(rlp.val_at(1)?)),
			Kind::HeaderProof => Ok(Response::HeaderProof(rlp.val_at(1)?)),
			Kind::TransactionIndex => Ok(Response::TransactionIndex(rlp.val_at(1)?)),
			Kind::Receipts => Ok(Response::Receipts(rlp.val_at(1)?)),
			Kind::Body => Ok(Response::Body(rlp.val_at(1)?)),
			Kind::Account => Ok(Response::Account(rlp.val_at(1)?)),
			Kind::Storage => Ok(Response::Storage(rlp.val_at(1)?)),
			Kind::Code => Ok(Response::Code(rlp.val_at(1)?)),
			Kind::Execution => Ok(Response::Execution(rlp.val_at(1)?)),
			Kind::Signal => Ok(Response::Signal(rlp.val_at(1)?)),
		}
	}
}

impl Encodable for Response {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);

		// hack around https://github.com/paritytech/parity-ethereum/issues/4356
		Encodable::rlp_append(&self.kind(), s);

		match *self {
			Response::Headers(ref res) => s.append(res),
			Response::HeaderProof(ref res) => s.append(res),
			Response::TransactionIndex(ref res) => s.append(res),
			Response::Receipts(ref res) => s.append(res),
			Response::Body(ref res) => s.append(res),
			Response::Account(ref res) => s.append(res),
			Response::Storage(ref res) => s.append(res),
			Response::Code(ref res) => s.append(res),
			Response::Execution(ref res) => s.append(res),
			Response::Signal(ref res) => s.append(res),
		};
	}
}

/// A potentially incomplete request.
pub trait IncompleteRequest: Sized {
	/// The complete variant of this request.
	type Complete;
	/// The response to this request.
	type Response: ResponseLike;

	/// Check prior outputs against the needed inputs.
	///
	/// This is called to ensure consistency of this request with
	/// others in the same packet.
	fn check_outputs<F>(&self, f: F) -> Result<(), NoSuchOutput>
		where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>;

	/// Note that this request will produce the following outputs.
	fn note_outputs<F>(&self, f: F) where F: FnMut(usize, OutputKind);

	/// Fill fields of the request.
	///
	/// This function is provided an "output oracle" which allows fetching of
	/// prior request outputs.
	/// Only outputs previously checked with `check_outputs` may be available.
	fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>;

	/// Attempt to convert this request into its complete variant.
	/// Will succeed if all fields have been filled, will fail otherwise.
	fn complete(self) -> Result<Self::Complete, NoSuchOutput>;

	/// Adjust back-reference request indices.
	fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize;
}

/// A request which can be checked against its response for more validity.
pub trait CheckedRequest: IncompleteRequest {
	/// Data extracted during the check.
	type Extract;
	/// Error encountered during the check.
	type Error;
	/// Environment passed to response check.
	type Environment;

	/// Check whether the response matches (beyond the type).
	fn check_response(&self, &Self::Complete, &Self::Environment, &Self::Response) -> Result<Self::Extract, Self::Error>;
}

/// A response-like object.
///
/// These contain re-usable outputs.
pub trait ResponseLike {
	/// Write all re-usable outputs into the provided function.
	fn fill_outputs<F>(&self, output_store: F) where F: FnMut(usize, Output);
}

/// Header request.
pub mod header {
	use super::{Field, HashOrNumber, NoSuchOutput, OutputKind, Output};
	use common_types::encoded;
	use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};

	/// Potentially incomplete headers request.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// Start block.
		pub start: Field<HashOrNumber>,
		/// Skip between.
		pub skip: u64,
		/// Maximum to return.
		pub max: u64,
		/// Whether to reverse from start.
		pub reverse: bool,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.start {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) =>
					f(req, idx, OutputKind::Hash).or_else(|_| f(req, idx, OutputKind::Number))
			}
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) { }

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.start {
				self.start = match oracle(req, idx) {
					Ok(Output::Hash(hash)) => Field::Scalar(hash.into()),
					Ok(Output::Number(num)) => Field::Scalar(num.into()),
					Err(_) => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				start: self.start.into_scalar()?,
				skip: self.skip,
				max: self.max,
				reverse: self.reverse,
			})
		}

		fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
			self.start.adjust_req(mapping)
		}
	}

	/// A complete header request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// Start block.
		pub start: HashOrNumber,
		/// Skip between.
		pub skip: u64,
		/// Maximum to return.
		pub max: u64,
		/// Whether to reverse from start.
		pub reverse: bool,
	}

	/// The output of a request for headers.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// The headers requested.
		pub headers: Vec<encoded::Header>,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by writing them into the function.
		fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) { }
	}

	impl Decodable for Response {
		fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
			use common_types::header::Header as FullHeader;

			let mut headers = Vec::new();

			for item in rlp.iter() {
				// check that it's a valid encoding.
				// TODO: just return full headers here?
				let _: FullHeader = item.as_val()?;
				headers.push(encoded::Header::new(item.as_raw().to_owned()));
			}

			Ok(Response { headers })
		}
	}

	impl Encodable for Response {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.begin_list(self.headers.len());
			for header in &self.headers {
				s.append_raw(header.rlp().as_raw(), 1);
			}
		}
	}
}

/// Request and response for header proofs.
pub mod header_proof {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};
	use ethereum_types::{H256, U256};
	use bytes::Bytes;

	/// Potentially incomplete header proof request.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// Block number.
		pub num: Field<u64>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.num {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) => f(req, idx, OutputKind::Number),
			}
		}

		fn note_outputs<F>(&self, mut note: F) where F: FnMut(usize, OutputKind) {
			note(0, OutputKind::Hash);
		}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.num {
				self.num = match oracle(req, idx) {
					Ok(Output::Number(num)) => Field::Scalar(num),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				num: self.num.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
			self.num.adjust_req(mapping)
		}
	}

	/// A complete header proof request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The number to get a header proof for.
		pub num: u64,
	}

	/// The output of a request for a header proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// Inclusion proof of the header and total difficulty in the CHT.
		pub proof: Vec<Bytes>,
		/// The proved header's hash.
		pub hash: H256,
		/// The proved header's total difficulty.
		pub td: U256,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
			f(0, Output::Hash(self.hash));
		}
	}

	impl Decodable for Response {
		fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
			Ok(Response {
				proof: rlp.list_at(0)?,
				hash: rlp.val_at(1)?,
				td: rlp.val_at(2)?,
			})
		}
	}

	impl Encodable for Response {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.begin_list(3)
				.append_list::<Vec<u8>,_>(&self.proof[..])
				.append(&self.hash)
				.append(&self.td);
		}
	}
}

/// Request and response for transaction index.
pub mod transaction_index {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethereum_types::H256;

	/// Potentially incomplete transaction index request.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// Transaction hash to get index for.
		pub hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.hash {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) => f(req, idx, OutputKind::Hash),
			}
		}

		fn note_outputs<F>(&self, mut f: F) where F: FnMut(usize, OutputKind) {
			f(0, OutputKind::Number);
			f(1, OutputKind::Hash);
		}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.hash {
				self.hash = match oracle(req, idx) {
					Ok(Output::Hash(hash)) => Field::Scalar(hash.into()),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				hash: self.hash.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
			self.hash.adjust_req(mapping)
		}
	}

	/// A complete transaction index request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The transaction hash to get index for.
		pub hash: H256,
	}

	/// The output of a request for transaction index.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Response {
		/// Block number.
		pub num: u64,
		/// Block hash
		pub hash: H256,
		/// Index in block.
		pub index: u64,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
			f(0, Output::Number(self.num));
			f(1, Output::Hash(self.hash));
		}
	}
}

/// Request and response for block receipts
pub mod block_receipts {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use common_types::receipt::Receipt;
	use ethereum_types::H256;

	/// Potentially incomplete block receipts request.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// Block hash to get receipts for.
		pub hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.hash {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) => f(req, idx, OutputKind::Hash),
			}
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.hash {
				self.hash = match oracle(req, idx) {
					Ok(Output::Hash(hash)) => Field::Scalar(hash.into()),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				hash: self.hash.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
			self.hash.adjust_req(mapping)
		}
	}

	/// A complete block receipts request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The number to get block receipts for.
		pub hash: H256,
	}

	/// The output of a request for block receipts.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodableWrapper, RlpDecodableWrapper)]
	pub struct Response {
		/// The block receipts.
		pub receipts: Vec<Receipt>
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}
}

/// Request and response for a block body
pub mod block_body {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use common_types::encoded;
	use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};
	use ethereum_types::H256;

	/// Potentially incomplete block body request.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// Block hash to get receipts for.
		pub hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.hash {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) => f(req, idx, OutputKind::Hash),
			}
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.hash {
				self.hash = match oracle(req, idx) {
					Ok(Output::Hash(hash)) => Field::Scalar(hash),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				hash: self.hash.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
			self.hash.adjust_req(mapping)
		}
	}

	/// A complete block body request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The hash to get a block body for.
		pub hash: H256,
	}

	/// The output of a request for block body.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// The block body.
		pub body: encoded::Body,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}

	impl Decodable for Response {
		fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
			use common_types::header::Header as FullHeader;
			use common_types::transaction::UnverifiedTransaction;

			// check body validity.
			let _: Vec<UnverifiedTransaction> = rlp.list_at(0)?;
			let _: Vec<FullHeader> = rlp.list_at(1)?;

			Ok(Response {
				body: encoded::Body::new(rlp.as_raw().to_owned()),
			})
		}
	}

	impl Encodable for Response {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.append_raw(&self.body.rlp().as_raw(), 1);
		}
	}
}

/// A request for an account proof.
pub mod account {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethereum_types::{H256, U256};
	use bytes::Bytes;

	/// Potentially incomplete request for an account proof.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// Block hash to request state proof for.
		pub block_hash: Field<H256>,
		/// Hash of the account's address.
		pub address_hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?
			}

			if let Field::BackReference(req, idx) = self.address_hash {
				f(req, idx, OutputKind::Hash)?
			}

			Ok(())
		}

		fn note_outputs<F>(&self, mut f: F) where F: FnMut(usize, OutputKind) {
			f(0, OutputKind::Hash);
			f(1, OutputKind::Hash);
		}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.block_hash {
				self.block_hash = match oracle(req, idx) {
					Ok(Output::Hash(block_hash)) => Field::Scalar(block_hash),
					_ => Field::BackReference(req, idx),
				}
			}

			if let Field::BackReference(req, idx) = self.address_hash {
				self.address_hash = match oracle(req, idx) {
					Ok(Output::Hash(address_hash)) => Field::Scalar(address_hash),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				block_hash: self.block_hash.into_scalar()?,
				address_hash: self.address_hash.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mut mapping: F) where F: FnMut(usize) -> usize {
			self.block_hash.adjust_req(&mut mapping);
			self.address_hash.adjust_req(&mut mapping);
		}
	}

	/// A complete request for an account.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// Block hash to request state proof for.
		pub block_hash: H256,
		/// Hash of the account's address.
		pub address_hash: H256,
	}

	/// The output of a request for an account state proof.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Response {
		/// Inclusion/exclusion proof
		pub proof: Vec<Bytes>,
		/// Account nonce.
		pub nonce: U256,
		/// Account balance.
		pub balance: U256,
		/// Account's code hash.
		pub code_hash: H256,
		/// Account's storage trie root.
		pub storage_root: H256,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
			f(0, Output::Hash(self.code_hash));
			f(1, Output::Hash(self.storage_root));
		}
	}
}

/// A request for a storage proof.
pub mod storage {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethereum_types::H256;
	use bytes::Bytes;

	/// Potentially incomplete request for an storage proof.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// Block hash to request state proof for.
		pub block_hash: Field<H256>,
		/// Hash of the account's address.
		pub address_hash: Field<H256>,
		/// Hash of the storage key.
		pub key_hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?
			}

			if let Field::BackReference(req, idx) = self.address_hash {
				f(req, idx, OutputKind::Hash)?
			}

			if let Field::BackReference(req, idx) = self.key_hash {
				f(req, idx, OutputKind::Hash)?
			}

			Ok(())
		}

		fn note_outputs<F>(&self, mut f: F) where F: FnMut(usize, OutputKind) {
			f(0, OutputKind::Hash);
		}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.block_hash {
				self.block_hash = match oracle(req, idx) {
					Ok(Output::Hash(block_hash)) => Field::Scalar(block_hash),
					_ => Field::BackReference(req, idx),
				}
			}

			if let Field::BackReference(req, idx) = self.address_hash {
				self.address_hash = match oracle(req, idx) {
					Ok(Output::Hash(address_hash)) => Field::Scalar(address_hash),
					_ => Field::BackReference(req, idx),
				}
			}

			if let Field::BackReference(req, idx) = self.key_hash {
				self.key_hash = match oracle(req, idx) {
					Ok(Output::Hash(key_hash)) => Field::Scalar(key_hash),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				block_hash: self.block_hash.into_scalar()?,
				address_hash: self.address_hash.into_scalar()?,
				key_hash: self.key_hash.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mut mapping: F) where F: FnMut(usize) -> usize {
			self.block_hash.adjust_req(&mut mapping);
			self.address_hash.adjust_req(&mut mapping);
			self.key_hash.adjust_req(&mut mapping);
		}
	}

	/// A complete request for a storage proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// Block hash to request state proof for.
		pub block_hash: H256,
		/// Hash of the account's address.
		pub address_hash: H256,
		/// Storage key hash.
		pub key_hash: H256,
	}

	/// The output of a request for an account state proof.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Response {
		/// Inclusion/exclusion proof
		pub proof: Vec<Bytes>,
		/// Storage value.
		pub value: H256,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
			f(0, Output::Hash(self.value));
		}
	}
}

/// A request for contract code.
pub mod contract_code {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethereum_types::H256;
	use bytes::Bytes;

	/// Potentially incomplete contract code request.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// The block hash to request the state for.
		pub block_hash: Field<H256>,
		/// The code hash.
		pub code_hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?;
			}
			if let Field::BackReference(req, idx) = self.code_hash {
				f(req, idx, OutputKind::Hash)?;
			}

			Ok(())
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.block_hash {
				self.block_hash = match oracle(req, idx) {
					Ok(Output::Hash(block_hash)) => Field::Scalar(block_hash),
					_ => Field::BackReference(req, idx),
				}
			}

			if let Field::BackReference(req, idx) = self.code_hash {
				self.code_hash = match oracle(req, idx) {
					Ok(Output::Hash(code_hash)) => Field::Scalar(code_hash),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				block_hash: self.block_hash.into_scalar()?,
				code_hash: self.code_hash.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mut mapping: F) where F: FnMut(usize) -> usize {
			self.block_hash.adjust_req(&mut mapping);
			self.code_hash.adjust_req(&mut mapping);
		}
	}

	/// A complete request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The block hash to request the state for.
		pub block_hash: H256,
		/// The code hash.
		pub code_hash: H256,
	}

	/// The output of a request for
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodableWrapper, RlpDecodableWrapper)]
	pub struct Response {
		/// The requested code.
		pub code: Bytes,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}
}

/// A request for proof of execution.
pub mod execution {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use common_types::transaction::Action;
	use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};
	use ethereum_types::{H256, U256, Address};
	use kvdb::DBValue;
	use bytes::Bytes;

	/// Potentially incomplete execution proof request.
	#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
	pub struct Incomplete {
		/// The block hash to request the state for.
		pub block_hash: Field<H256>,
		/// The address the transaction should be from.
		pub from: Address,
		/// The action of the transaction.
		pub action: Action,
		/// The amount of gas to prove.
		pub gas: U256,
		/// The gas price.
		pub gas_price: U256,
		/// The value to transfer.
		pub value: U256,
		/// Call data.
		pub data: Bytes,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?;
			}

			Ok(())
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.block_hash {
				self.block_hash = match oracle(req, idx) {
					Ok(Output::Hash(block_hash)) => Field::Scalar(block_hash),
					_ => Field::BackReference(req, idx),
				}
			}
		}
		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				block_hash: self.block_hash.into_scalar()?,
				from: self.from,
				action: self.action,
				gas: self.gas,
				gas_price: self.gas_price,
				value: self.value,
				data: self.data,
			})
		}

		fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
			self.block_hash.adjust_req(mapping);
		}
	}

	/// A complete request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The block hash to request the state for.
		pub block_hash: H256,
		/// The address the transaction should be from.
		pub from: Address,
		/// The action of the transaction.
		pub action: Action,
		/// The amount of gas to prove.
		pub gas: U256,
		/// The gas price.
		pub gas_price: U256,
		/// The value to transfer.
		pub value: U256,
		/// Call data.
		pub data: Bytes,
	}

	/// The output of a request for proof of execution
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// All state items (trie nodes, code) necessary to re-prove the transaction.
		pub items: Vec<DBValue>,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}

	impl Decodable for Response {
		fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
			let mut items = Vec::new();
			for raw_item in rlp.iter() {
				let mut item = DBValue::new();
				item.append_slice(raw_item.data()?);
				items.push(item);
			}

			Ok(Response { items })
		}
	}

	impl Encodable for Response {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.begin_list(self.items.len());

			for item in &self.items {
				s.append(&&**item);
			}
		}
	}
}

/// A request for epoch signal data.
pub mod epoch_signal {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};
	use ethereum_types::H256;
	use bytes::Bytes;

	/// Potentially incomplete epoch signal request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// The block hash to request the signal for.
		pub block_hash: Field<H256>,
	}

	impl Decodable for Incomplete {
		fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
			Ok(Incomplete {
				block_hash: rlp.val_at(0)?,
			})
		}
	}

	impl Encodable for Incomplete {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.begin_list(1).append(&self.block_hash);
		}
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;
		type Response = Response;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?;
			}

			Ok(())
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
			if let Field::BackReference(req, idx) = self.block_hash {
				self.block_hash = match oracle(req, idx) {
					Ok(Output::Hash(block_hash)) => Field::Scalar(block_hash),
					_ => Field::BackReference(req, idx),
				}
			}
		}

		fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
			Ok(Complete {
				block_hash: self.block_hash.into_scalar()?,
			})
		}

		fn adjust_refs<F>(&mut self, mut mapping: F) where F: FnMut(usize) -> usize {
			self.block_hash.adjust_req(&mut mapping);
		}
	}

	/// A complete request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The block hash to request the epoch signal for.
		pub block_hash: H256,
	}

	/// The output of a request for an epoch signal.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// The requested epoch signal.
		pub signal: Bytes,
	}

	impl super::ResponseLike for Response {
		/// Fill reusable outputs by providing them to the function.
		fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}

	impl Decodable for Response {
		fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {

			Ok(Response {
				signal: rlp.as_val()?,
			})
		}
	}

	impl Encodable for Response {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.append(&self.signal);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common_types::header::Header;

	fn check_roundtrip<T>(val: T)
		where T: ::rlp::Encodable + ::rlp::Decodable + PartialEq + ::std::fmt::Debug
	{
		// check as single value.
		let bytes = ::rlp::encode(&val);
		let new_val: T = ::rlp::decode(&bytes).unwrap();
		assert_eq!(val, new_val);

		// check as list containing single value.
		let list = [val];
		let bytes = ::rlp::encode_list(&list);
		let new_list: Vec<T> = ::rlp::decode_list(&bytes);
		assert_eq!(&list, &new_list[..]);
	}

	#[test]
	fn hash_or_number_roundtrip() {
		let hash = HashOrNumber::Hash(H256::default());
		let number = HashOrNumber::Number(5);

		check_roundtrip(hash);
		check_roundtrip(number);
	}

	#[test]
	fn field_roundtrip() {
		let field_scalar = Field::Scalar(5usize);
		let field_back: Field<usize> = Field::BackReference(1, 2);

		check_roundtrip(field_scalar);
		check_roundtrip(field_back);
	}

	#[test]
	fn headers_roundtrip() {
		let req = IncompleteHeadersRequest {
			start: Field::Scalar(5u64.into()),
			skip: 0,
			max: 100,
			reverse: false,
		};

		let full_req = Request::Headers(req.clone());
		let res = HeadersResponse {
			headers: vec![
				::common_types::encoded::Header::new(::rlp::encode(&Header::default()))
			]
		};
		let full_res = Response::Headers(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn header_proof_roundtrip() {
		let req = IncompleteHeaderProofRequest {
			num: Field::BackReference(1, 234),
		};

		let full_req = Request::HeaderProof(req.clone());
		let res = HeaderProofResponse {
			proof: vec![vec![1, 2, 3], vec![4, 5, 6]],
			hash: Default::default(),
			td: 100.into(),
		};
		let full_res = Response::HeaderProof(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn transaction_index_roundtrip() {
		let req = IncompleteTransactionIndexRequest {
			hash: Field::Scalar(Default::default()),
		};

		let full_req = Request::TransactionIndex(req.clone());
		let res = TransactionIndexResponse {
			num: 1000,
			hash: ::ethereum_types::H256::random(),
			index: 4,
		};
		let full_res = Response::TransactionIndex(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn receipts_roundtrip() {
		use common_types::receipt::{Receipt, TransactionOutcome};
		let req = IncompleteReceiptsRequest {
			hash: Field::Scalar(Default::default()),
		};

		let full_req = Request::Receipts(req.clone());
		let receipt = Receipt::new(TransactionOutcome::Unknown, Default::default(), Vec::new());
		let res = ReceiptsResponse {
			receipts: vec![receipt.clone(), receipt],
		};
		let full_res = Response::Receipts(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn body_roundtrip() {
		use common_types::transaction::{Transaction, UnverifiedTransaction};
		let req = IncompleteBodyRequest {
			hash: Field::Scalar(Default::default()),
		};

		let full_req = Request::Body(req.clone());
		let res = BodyResponse {
			body: {
				let header = ::common_types::header::Header::default();
				let tx = UnverifiedTransaction::from(Transaction::default().fake_sign(Default::default()));
				let mut stream = RlpStream::new_list(2);
				stream.begin_list(2).append(&tx).append(&tx)
					.begin_list(1).append(&header);

				::common_types::encoded::Body::new(stream.out())
			},
		};
		let full_res = Response::Body(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn account_roundtrip() {
		let req = IncompleteAccountRequest {
			block_hash: Field::Scalar(Default::default()),
			address_hash: Field::BackReference(1, 2),
		};

		let full_req = Request::Account(req.clone());
		let res = AccountResponse {
			proof: vec![vec![1, 2, 3], vec![4, 5, 6]],
			nonce: 100.into(),
			balance: 123456.into(),
			code_hash: Default::default(),
			storage_root: Default::default(),
		};
		let full_res = Response::Account(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn storage_roundtrip() {
		let req = IncompleteStorageRequest {
			block_hash: Field::Scalar(Default::default()),
			address_hash: Field::BackReference(1, 2),
			key_hash: Field::BackReference(3, 2),
		};

		let full_req = Request::Storage(req.clone());
		let res = StorageResponse {
			proof: vec![vec![1, 2, 3], vec![4, 5, 6]],
			value: H256::default(),
		};
		let full_res = Response::Storage(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn code_roundtrip() {
		let req = IncompleteCodeRequest {
			block_hash: Field::Scalar(Default::default()),
			code_hash: Field::BackReference(3, 2),
		};

		let full_req = Request::Code(req.clone());
		let res = CodeResponse {
			code: vec![1, 2, 3, 4, 5, 6, 7, 6, 5, 4],
		};
		let full_res = Response::Code(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn execution_roundtrip() {
		use kvdb::DBValue;

		let req = IncompleteExecutionRequest {
			block_hash: Field::Scalar(Default::default()),
			from: Default::default(),
			action: ::common_types::transaction::Action::Create,
			gas: 100_000.into(),
			gas_price: 0.into(),
			value: 100_000_001.into(),
			data: vec![1, 2, 3, 2, 1],
		};

		let full_req = Request::Execution(req.clone());
		let res = ExecutionResponse {
			items: vec![DBValue::new(), {
				let mut value = DBValue::new();
				value.append_slice(&[1, 1, 1, 2, 3]);
				value
			}],
		};
		let full_res = Response::Execution(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}

	#[test]
	fn vec_test() {
		use rlp::*;

		let reqs: Vec<_> = (0..10).map(|_| IncompleteExecutionRequest {
			block_hash: Field::Scalar(Default::default()),
			from: Default::default(),
			action: ::common_types::transaction::Action::Create,
			gas: 100_000.into(),
			gas_price: 0.into(),
			value: 100_000_001.into(),
			data: vec![1, 2, 3, 2, 1],
		}).map(Request::Execution).collect();

		let mut stream = RlpStream::new_list(2);
		stream.append(&100usize).append_list(&reqs);
		let out = stream.out();

		let rlp = Rlp::new(&out);
		assert_eq!(rlp.val_at::<usize>(0).unwrap(), 100usize);
		assert_eq!(rlp.list_at::<Request>(1).unwrap(), reqs);
	}

	#[test]
	fn responses_vec() {
		use common_types::receipt::{Receipt, TransactionOutcome};
		let mut stream = RlpStream::new_list(2);
				stream.begin_list(0).begin_list(0);

		let body = ::common_types::encoded::Body::new(stream.out());
		let reqs = vec![
			Response::Headers(HeadersResponse { headers: vec![] }),
			Response::HeaderProof(HeaderProofResponse { proof: vec![], hash: Default::default(), td: 100.into()}),
			Response::Receipts(ReceiptsResponse { receipts: vec![Receipt::new(TransactionOutcome::Unknown, Default::default(), Vec::new())] }),
			Response::Body(BodyResponse { body: body }),
			Response::Account(AccountResponse {
				proof: vec![],
				nonce: 100.into(),
				balance: 123.into(),
				code_hash: Default::default(),
				storage_root: Default::default()
			}),
			Response::Storage(StorageResponse { proof: vec![], value: H256::default() }),
			Response::Code(CodeResponse { code: vec![1, 2, 3, 4, 5] }),
			Response::Execution(ExecutionResponse { items: vec![] }),
		];

		let raw = ::rlp::encode_list(&reqs);
		assert_eq!(::rlp::decode_list::<Response>(&raw), reqs);
	}

	#[test]
	fn epoch_signal_roundtrip() {
		let req = IncompleteSignalRequest {
			block_hash: Field::Scalar(Default::default()),
		};

		let full_req = Request::Signal(req.clone());
		let res = SignalResponse {
			signal: vec![1, 2, 3, 4, 5, 6, 7, 6, 5, 4],
		};
		let full_res = Response::Signal(res.clone());

		check_roundtrip(req);
		check_roundtrip(full_req);
		check_roundtrip(res);
		check_roundtrip(full_res);
	}
}
