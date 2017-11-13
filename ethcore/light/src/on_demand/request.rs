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

//! Request types, verification, and verification errors.

use std::sync::Arc;

use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::engines::{EthEngine, StateDependentProof};
use ethcore::machine::EthereumMachine;
use ethcore::receipt::Receipt;
use ethcore::state::{self, ProvedExecution};
use ethcore::transaction::SignedTransaction;
use vm::EnvInfo;
use hash::{KECCAK_NULL_RLP, KECCAK_EMPTY, KECCAK_EMPTY_LIST_RLP, keccak};

use request::{self as net_request, IncompleteRequest, CompleteRequest, Output, OutputKind, Field};

use rlp::{RlpStream, UntrustedRlp};
use bigint::prelude::U256;
use bigint::hash::H256;
use parking_lot::Mutex;
use util::{Address, DBValue, HashDB};
use bytes::Bytes;
use memorydb::MemoryDB;
use trie::{Trie, TrieDB, TrieError};

const SUPPLIED_MATCHES: &'static str = "supplied responses always match produced requests; enforced by `check_response`; qed";

/// Core unit of the API: submit batches of these to be answered with `Response`s.
#[derive(Clone)]
pub enum Request {
	/// A request for a header proof.
	HeaderProof(HeaderProof),
	/// A request for a header by hash.
	HeaderByHash(HeaderByHash),
	/// A request for the index of a transaction.
	TransactionIndex(TransactionIndex),
	/// A request for block receipts.
	Receipts(BlockReceipts),
	/// A request for a block body.
	Body(Body),
	/// A request for an account.
	Account(Account),
	/// A request for a contract's code.
	Code(Code),
	/// A request for proof of execution.
	Execution(TransactionProof),
	/// A request for epoch change signal.
	Signal(Signal),
}

/// A request argument.
pub trait RequestArg {
	/// the response type.
	type Out;

	/// Create the request type.
	/// `extract` must not fail when presented with the corresponding
	/// `Response`.
	fn make(self) -> Request;

	/// May not panic if the response corresponds with the request
	/// from `make`.
	/// Is free to panic otherwise.
	fn extract(r: Response) -> Self::Out;
}

/// An adapter can be thought of as a grouping of request argument types.
/// This is implemented for various tuples and convenient types.
pub trait RequestAdapter {
	/// The output type.
	type Out;

	/// Infallibly produce requests. When `extract_from` is presented
	/// with the corresponding response vector, it may not fail.
	fn make_requests(self) -> Vec<Request>;

	/// Extract the output type from the given responses.
	/// If they are the corresponding responses to the requests
	/// made by `make_requests`, do not panic.
	fn extract_from(Vec<Response>) -> Self::Out;
}

impl<T: RequestArg> RequestAdapter for Vec<T> {
	type Out = Vec<T::Out>;

	fn make_requests(self) -> Vec<Request> {
		self.into_iter().map(RequestArg::make).collect()
	}

	fn extract_from(r: Vec<Response>) -> Self::Out {
		r.into_iter().map(T::extract).collect()
	}
}

// helper to implement `RequestArg` and `From` for a single request kind.
macro_rules! impl_single {
	($variant: ident, $me: ty, $out: ty) => {
		impl RequestArg for $me {
			type Out = $out;

			fn make(self) -> Request {
				Request::$variant(self)
			}

			fn extract(r: Response) -> $out {
				match r {
					Response::$variant(x) => x,
					_ => panic!(SUPPLIED_MATCHES),
				}
			}
		}

		impl From<$me> for Request {
			fn from(me: $me) -> Request {
				Request::$variant(me)
			}
		}
	}
}

// implement traits for each kind of request.
impl_single!(HeaderProof, HeaderProof, (H256, U256));
impl_single!(HeaderByHash, HeaderByHash, encoded::Header);
impl_single!(TransactionIndex, TransactionIndex, net_request::TransactionIndexResponse);
impl_single!(Receipts, BlockReceipts, Vec<Receipt>);
impl_single!(Body, Body, encoded::Block);
impl_single!(Account, Account, Option<BasicAccount>);
impl_single!(Code, Code, Bytes);
impl_single!(Execution, TransactionProof, super::ExecutionResult);
impl_single!(Signal, Signal, Vec<u8>);

macro_rules! impl_args {
	() => {
		impl<T: RequestArg> RequestAdapter for T {
			type Out = T::Out;

			fn make_requests(self) -> Vec<Request> {
				vec![self.make()]
			}

			fn extract_from(mut responses: Vec<Response>) -> Self::Out {
				T::extract(responses.pop().expect(SUPPLIED_MATCHES))
			}
		}
	};
	($first: ident, $($next: ident,)*) => {
		impl<
			$first: RequestArg,
			$($next: RequestArg,)*
		>
		RequestAdapter for ($first, $($next,)*) {
			type Out = ($first::Out, $($next::Out,)*);

			fn make_requests(self) -> Vec<Request> {
				let ($first, $($next,)*) = self;

				vec![
					$first.make(),
					$($next.make(),)*
				]
			}

			fn extract_from(responses: Vec<Response>) -> Self::Out {
				let mut iter = responses.into_iter();
				(
					$first::extract(iter.next().expect(SUPPLIED_MATCHES)),
					$($next::extract(iter.next().expect(SUPPLIED_MATCHES)),)*
				)
			}
		}
		impl_args!($($next,)*);
	}
}

mod impls {
	#![allow(non_snake_case)]

	use super::{RequestAdapter, RequestArg, Request, Response, SUPPLIED_MATCHES};

	impl_args!(A, B, C, D, E, F, G, H, I, J, K, L,);
}

/// A block header to be used for verification.
/// May be stored or an unresolved output of a prior request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderRef {
	/// A stored header.
	Stored(encoded::Header),
	/// An unresolved header. The first item here is the index of the request which
	/// will return the header. The second is a back-reference pointing to a block hash
	/// which can be used to make requests until that header is resolved.
	Unresolved(usize, Field<H256>),
}

impl HeaderRef {
	/// Attempt to inspect the header.
	pub fn as_ref(&self) -> Result<&encoded::Header, Error> {
		match *self {
			HeaderRef::Stored(ref hdr) => Ok(hdr),
			HeaderRef::Unresolved(idx, _) => Err(Error::UnresolvedHeader(idx)),
		}
	}

	// get the blockhash field to be used in requests.
	fn field(&self) -> Field<H256> {
		match *self {
			HeaderRef::Stored(ref hdr) => Field::Scalar(hdr.hash()),
			HeaderRef::Unresolved(_, ref field) => field.clone(),
		}
	}

	// yield the index of the request which will produce the header.
	fn needs_header(&self) -> Option<(usize, Field<H256>)> {
		match *self {
			HeaderRef::Stored(_) => None,
			HeaderRef::Unresolved(idx, ref field) => Some((idx, field.clone())),
		}
	}
}

impl From<encoded::Header> for HeaderRef {
	fn from(header: encoded::Header) -> Self {
		HeaderRef::Stored(header)
	}
}

/// Requests coupled with their required data for verification.
/// This is used internally but not part of the public API.
#[derive(Clone)]
#[allow(missing_docs)]
pub enum CheckedRequest {
	HeaderProof(HeaderProof, net_request::IncompleteHeaderProofRequest),
	HeaderByHash(HeaderByHash, net_request::IncompleteHeadersRequest),
	TransactionIndex(TransactionIndex, net_request::IncompleteTransactionIndexRequest),
	Receipts(BlockReceipts, net_request::IncompleteReceiptsRequest),
	Body(Body, net_request::IncompleteBodyRequest),
	Account(Account, net_request::IncompleteAccountRequest),
	Code(Code, net_request::IncompleteCodeRequest),
	Execution(TransactionProof, net_request::IncompleteExecutionRequest),
	Signal(Signal, net_request::IncompleteSignalRequest)
}

impl From<Request> for CheckedRequest {
	fn from(req: Request) -> Self {
		match req {
			Request::HeaderByHash(req) => {
				let net_req = net_request::IncompleteHeadersRequest {
					start: req.0.map(Into::into),
					skip: 0,
					max: 1,
					reverse: false,
				};
				CheckedRequest::HeaderByHash(req, net_req)
			}
			Request::HeaderProof(req) => {
				let net_req = net_request::IncompleteHeaderProofRequest {
					num: req.num().into(),
				};
				CheckedRequest::HeaderProof(req, net_req)
			}
			Request::TransactionIndex(req) => {
				let net_req = net_request::IncompleteTransactionIndexRequest {
					hash: req.0.clone(),
				};
				CheckedRequest::TransactionIndex(req, net_req)
			}
			Request::Body(req) =>  {
				let net_req = net_request::IncompleteBodyRequest {
					hash: req.0.field(),
				};
				CheckedRequest::Body(req, net_req)
			}
			Request::Receipts(req) => {
				let net_req = net_request::IncompleteReceiptsRequest {
					hash: req.0.field(),
				};
				CheckedRequest::Receipts(req, net_req)
			}
			Request::Account(req) => {
				let net_req = net_request::IncompleteAccountRequest {
					block_hash: req.header.field(),
					address_hash: ::hash::keccak(&req.address).into(),
				};
				CheckedRequest::Account(req, net_req)
			}
			Request::Code(req) => {
				let net_req = net_request::IncompleteCodeRequest {
					block_hash: req.header.field(),
					code_hash: req.code_hash.into(),
				};
				CheckedRequest::Code(req, net_req)
			}
			Request::Execution(req) => {
				let net_req = net_request::IncompleteExecutionRequest {
					block_hash: req.header.field(),
					from: req.tx.sender(),
					gas: req.tx.gas,
					gas_price: req.tx.gas_price,
					action: req.tx.action.clone(),
					value: req.tx.value,
					data: req.tx.data.clone(),
				};
				CheckedRequest::Execution(req, net_req)
			}
			Request::Signal(req) => {
				let net_req = net_request::IncompleteSignalRequest {
					block_hash: req.hash.into(),
				};
				CheckedRequest::Signal(req, net_req)
			}
		}
	}
}

impl CheckedRequest {
	/// Convert this into a network request.
	pub fn into_net_request(self) -> net_request::Request {
		use ::request::Request as NetRequest;

		match self {
			CheckedRequest::HeaderProof(_, req) => NetRequest::HeaderProof(req),
			CheckedRequest::HeaderByHash(_, req) => NetRequest::Headers(req),
			CheckedRequest::TransactionIndex(_, req) => NetRequest::TransactionIndex(req),
			CheckedRequest::Receipts(_, req) => NetRequest::Receipts(req),
			CheckedRequest::Body(_, req) => NetRequest::Body(req),
			CheckedRequest::Account(_, req) => NetRequest::Account(req),
			CheckedRequest::Code(_, req) => NetRequest::Code(req),
			CheckedRequest::Execution(_, req) => NetRequest::Execution(req),
			CheckedRequest::Signal(_, req) => NetRequest::Signal(req),
		}
	}

	/// Whether this needs a header from a prior request.
	/// Returns `Some` with the index of the request returning the header
	/// and the field giving the hash
	/// if so, `None` otherwise.
	pub fn needs_header(&self) -> Option<(usize, Field<H256>)> {
		match *self {
			CheckedRequest::Receipts(ref x, _) => x.0.needs_header(),
			CheckedRequest::Body(ref x, _) => x.0.needs_header(),
			CheckedRequest::Account(ref x, _) => x.header.needs_header(),
			CheckedRequest::Code(ref x, _) => x.header.needs_header(),
			CheckedRequest::Execution(ref x, _) => x.header.needs_header(),
			_ => None,
		}
	}

	/// Provide a header where one was needed. Should only be called if `needs_header`
	/// returns `Some`, and for correctness, only use the header yielded by the correct
	/// request.
	pub fn provide_header(&mut self, header: encoded::Header) {
		match *self {
			CheckedRequest::Receipts(ref mut x, _) => x.0 = HeaderRef::Stored(header),
			CheckedRequest::Body(ref mut x, _) => x.0 = HeaderRef::Stored(header),
			CheckedRequest::Account(ref mut x, _) => x.header = HeaderRef::Stored(header),
			CheckedRequest::Code(ref mut x, _) => x.header = HeaderRef::Stored(header),
			CheckedRequest::Execution(ref mut x, _) => x.header = HeaderRef::Stored(header),
			_ => {},
		}
	}

	/// Attempt to complete the request based on data in the cache.
	pub fn respond_local(&self, cache: &Mutex<::cache::Cache>) -> Option<Response> {
		match *self {
			CheckedRequest::HeaderProof(ref check, _) => {
				let mut cache = cache.lock();
				cache.block_hash(&check.num)
					.and_then(|h| cache.chain_score(&h).map(|s| (h, s)))
					.map(|(h, s)| Response::HeaderProof((h, s)))
			}
			CheckedRequest::HeaderByHash(_, ref req) => {
				if let Some(&net_request::HashOrNumber::Hash(ref h)) = req.start.as_ref() {
					return cache.lock().block_header(h).map(Response::HeaderByHash);
				}

				None
			}
			CheckedRequest::Receipts(ref check, ref req) => {
				// empty transactions -> no receipts
				if check.0.as_ref().ok().map_or(false, |hdr| hdr.receipts_root() == KECCAK_NULL_RLP) {
					return Some(Response::Receipts(Vec::new()));
				}

				req.hash.as_ref()
					.and_then(|hash| cache.lock().block_receipts(hash))
					.map(Response::Receipts)
			}
			CheckedRequest::Body(ref check, ref req) => {
				// check for empty body.
				if let Some(hdr) = check.0.as_ref().ok() {
					if hdr.transactions_root() == KECCAK_NULL_RLP && hdr.uncles_hash() == KECCAK_EMPTY_LIST_RLP {
						let mut stream = RlpStream::new_list(3);
						stream.append_raw(hdr.rlp().as_raw(), 1);
						stream.begin_list(0);
						stream.begin_list(0);

						return Some(Response::Body(encoded::Block::new(stream.out())));
					}
				}

				// otherwise, check for cached body and header.
				let block_hash = req.hash.as_ref()
					.cloned()
					.or_else(|| check.0.as_ref().ok().map(|hdr| hdr.hash()));
				let block_hash = match block_hash {
					Some(hash) => hash,
					None => return None,
				};

				let mut cache = cache.lock();
				let cached_header;

				// can't use as_ref here although it seems like you would be able to:
				// it complains about uninitialized `cached_header`.
				let block_header = match check.0.as_ref().ok() {
					Some(hdr) => Some(hdr),
					None => {
						cached_header = cache.block_header(&block_hash);
						cached_header.as_ref()
					}
				};

				block_header
					.and_then(|hdr| cache.block_body(&block_hash).map(|b| (hdr, b)))
					.map(|(hdr, body)| {
						let mut stream = RlpStream::new_list(3);
						let body = body.rlp();
						stream.append_raw(&hdr.rlp().as_raw(), 1);
						stream.append_raw(&body.at(0).as_raw(), 1);
						stream.append_raw(&body.at(1).as_raw(), 1);

						Response::Body(encoded::Block::new(stream.out()))
					})
			}
			CheckedRequest::Code(_, ref req) => {
				if req.code_hash.as_ref().map_or(false, |&h| h == KECCAK_EMPTY) {
					Some(Response::Code(Vec::new()))
				} else {
					None
				}
			}
			_ => None,
		}
	}
}

macro_rules! match_me {
	($me: expr, ($check: pat, $req: pat) => $e: expr) => {
		match $me {
			CheckedRequest::HeaderProof($check, $req) => $e,
			CheckedRequest::HeaderByHash($check, $req) => $e,
			CheckedRequest::TransactionIndex($check, $req) => $e,
			CheckedRequest::Receipts($check, $req) => $e,
			CheckedRequest::Body($check, $req) => $e,
			CheckedRequest::Account($check, $req) => $e,
			CheckedRequest::Code($check, $req) => $e,
			CheckedRequest::Execution($check, $req) => $e,
			CheckedRequest::Signal($check, $req) => $e,
		}
	}
}

impl IncompleteRequest for CheckedRequest {
	type Complete = CompleteRequest;
	type Response = net_request::Response;

	fn check_outputs<F>(&self, mut f: F) -> Result<(), net_request::NoSuchOutput>
		where F: FnMut(usize, usize, OutputKind) -> Result<(), net_request::NoSuchOutput>
	{
		match *self {
			CheckedRequest::HeaderProof(_, ref req) => req.check_outputs(f),
			CheckedRequest::HeaderByHash(ref check, ref req) => {
				req.check_outputs(&mut f)?;

				// make sure the output given is definitively a hash.
				match check.0 {
					Field::BackReference(r, idx) => f(r, idx, OutputKind::Hash),
					_ => Ok(()),
				}
			}
			CheckedRequest::TransactionIndex(_, ref req) => req.check_outputs(f),
			CheckedRequest::Receipts(_, ref req) => req.check_outputs(f),
			CheckedRequest::Body(_, ref req) => req.check_outputs(f),
			CheckedRequest::Account(_, ref req) => req.check_outputs(f),
			CheckedRequest::Code(_, ref req) => req.check_outputs(f),
			CheckedRequest::Execution(_, ref req) => req.check_outputs(f),
			CheckedRequest::Signal(_, ref req) => req.check_outputs(f),
		}
	}

	fn note_outputs<F>(&self, f: F) where F: FnMut(usize, OutputKind) {
		match_me!(*self, (_, ref req) => req.note_outputs(f))
	}

	fn fill<F>(&mut self, f: F) where F: Fn(usize, usize) -> Result<Output, net_request::NoSuchOutput> {
		match_me!(*self, (_, ref mut req) => req.fill(f))
	}

	fn complete(self) -> Result<Self::Complete, net_request::NoSuchOutput> {
		match self {
			CheckedRequest::HeaderProof(_, req) => req.complete().map(CompleteRequest::HeaderProof),
			CheckedRequest::HeaderByHash(_, req) => req.complete().map(CompleteRequest::Headers),
			CheckedRequest::TransactionIndex(_, req) => req.complete().map(CompleteRequest::TransactionIndex),
			CheckedRequest::Receipts(_, req) => req.complete().map(CompleteRequest::Receipts),
			CheckedRequest::Body(_, req) => req.complete().map(CompleteRequest::Body),
			CheckedRequest::Account(_, req) => req.complete().map(CompleteRequest::Account),
			CheckedRequest::Code(_, req) => req.complete().map(CompleteRequest::Code),
			CheckedRequest::Execution(_, req) => req.complete().map(CompleteRequest::Execution),
			CheckedRequest::Signal(_, req) => req.complete().map(CompleteRequest::Signal),
		}
	}


	fn adjust_refs<F>(&mut self, mapping: F) where F: FnMut(usize) -> usize {
		match_me!(*self, (_, ref mut req) => req.adjust_refs(mapping))
	}
}

impl net_request::CheckedRequest for CheckedRequest {
	type Extract = Response;
	type Error = Error;
	type Environment = Mutex<::cache::Cache>;

	/// Check whether the response matches (beyond the type).
	fn check_response(&self, complete: &Self::Complete, cache: &Mutex<::cache::Cache>, response: &Self::Response) -> Result<Response, Error> {
		use ::request::Response as NetResponse;

		// helper for expecting a specific response for a given request.
		macro_rules! expect {
			($res: pat => $e: expr) => {{
				match (response, complete) {
					$res => $e,
					_ => Err(Error::WrongKind),
				}
			}}
		}

		// check response against contained prover.
		match *self {
			CheckedRequest::HeaderProof(ref prover, _) =>
				expect!((&NetResponse::HeaderProof(ref res), _) =>
					prover.check_response(cache, &res.proof).map(Response::HeaderProof)),
			CheckedRequest::HeaderByHash(ref prover, _) =>
				expect!((&NetResponse::Headers(ref res), &CompleteRequest::Headers(ref req)) =>
					prover.check_response(cache, &req.start, &res.headers).map(Response::HeaderByHash)),
			CheckedRequest::TransactionIndex(ref prover, _) =>
				expect!((&NetResponse::TransactionIndex(ref res), _) =>
					prover.check_response(cache, res).map(Response::TransactionIndex)),
			CheckedRequest::Receipts(ref prover, _) =>
				expect!((&NetResponse::Receipts(ref res), _) =>
					prover.check_response(cache, &res.receipts).map(Response::Receipts)),
			CheckedRequest::Body(ref prover, _) =>
				expect!((&NetResponse::Body(ref res), _) =>
					prover.check_response(cache, &res.body).map(Response::Body)),
			CheckedRequest::Account(ref prover, _) =>
				expect!((&NetResponse::Account(ref res), _) =>
					prover.check_response(cache, &res.proof).map(Response::Account)),
			CheckedRequest::Code(ref prover, _) =>
				expect!((&NetResponse::Code(ref res), &CompleteRequest::Code(ref req)) =>
					prover.check_response(cache, &req.code_hash, &res.code).map(Response::Code)),
			CheckedRequest::Execution(ref prover, _) =>
				expect!((&NetResponse::Execution(ref res), _) =>
					prover.check_response(cache, &res.items).map(Response::Execution)),
			CheckedRequest::Signal(ref prover, _) =>
				expect!((&NetResponse::Signal(ref res), _) =>
					prover.check_response(cache, &res.signal).map(Response::Signal)),
		}
	 }
}

/// Responses to on-demand requests.
/// All of these are checked.
pub enum Response {
	/// Response to a header proof request.
	/// Returns the hash and chain score.
	HeaderProof((H256, U256)),
	/// Response to a header-by-hash request.
	HeaderByHash(encoded::Header),
	/// Response to a transaction-index request.
	TransactionIndex(net_request::TransactionIndexResponse),
	/// Response to a receipts request.
	Receipts(Vec<Receipt>),
	/// Response to a block body request.
	Body(encoded::Block),
	/// Response to an Account request.
	// TODO: `unwrap_or(engine_defaults)`
	Account(Option<BasicAccount>),
	/// Response to a request for code.
	Code(Vec<u8>),
	/// Response to a request for proved execution.
	Execution(super::ExecutionResult),
	/// Response to a request for epoch change signal.
	Signal(Vec<u8>),
}

impl net_request::ResponseLike for Response {
	fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
		match *self {
			Response::HeaderProof((ref hash, _)) => f(0, Output::Hash(*hash)),
			Response::Account(None) => {
				f(0, Output::Hash(KECCAK_EMPTY)); // code hash
				f(1, Output::Hash(KECCAK_NULL_RLP)); // storage root.
			}
			Response::Account(Some(ref acc)) => {
				f(0, Output::Hash(acc.code_hash));
				f(1, Output::Hash(acc.storage_root));
			}
			_ => {}
		}
	}
}

/// Errors in verification.
#[derive(Debug, PartialEq)]
pub enum Error {
	/// RLP decoder error.
	Decoder(::rlp::DecoderError),
	/// Empty response.
	Empty,
	/// Trie lookup error (result of bad proof)
	Trie(TrieError),
	/// Bad inclusion proof
	BadProof,
	/// Header by number instead of hash.
	HeaderByNumber,
	/// Unresolved header reference.
	UnresolvedHeader(usize),
	/// Wrong header number.
	WrongNumber(u64, u64),
	/// Wrong hash.
	WrongHash(H256, H256),
	/// Wrong trie root.
	WrongTrieRoot(H256, H256),
	/// Wrong response kind.
	WrongKind,
}

impl From<::rlp::DecoderError> for Error {
	fn from(err: ::rlp::DecoderError) -> Self {
		Error::Decoder(err)
	}
}

impl From<Box<TrieError>> for Error {
	fn from(err: Box<TrieError>) -> Self {
		Error::Trie(*err)
	}
}

/// Request for header proof by number
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderProof {
	/// The header's number.
	num: u64,
	/// The cht number for the given block number.
	cht_num: u64,
	/// The root of the CHT containing this header.
	cht_root: H256,
}

impl HeaderProof {
	/// Construct a new header-by-number request. Fails if the given number is 0.
	/// Provide the expected CHT root to compare against.
	pub fn new(num: u64, cht_root: H256) -> Option<Self> {
		::cht::block_to_cht_number(num).map(|cht_num| HeaderProof {
			num: num,
			cht_num: cht_num,
			cht_root: cht_root,
		})
	}

	/// Access the requested block number.
	pub fn num(&self) -> u64 { self.num }

	/// Access the CHT number.
	pub fn cht_num(&self) -> u64 { self.cht_num }

	/// Access the expected CHT root.
	pub fn cht_root(&self) -> H256 { self.cht_root }

	/// Check a response with a CHT proof, get a hash and total difficulty back.
	pub fn check_response(&self, cache: &Mutex<::cache::Cache>, proof: &[Bytes]) -> Result<(H256, U256), Error> {
		match ::cht::check_proof(proof, self.num, self.cht_root) {
			Some((expected_hash, td)) => {
				let mut cache = cache.lock();
				cache.insert_block_hash(self.num, expected_hash);
				cache.insert_chain_score(expected_hash, td);

				Ok((expected_hash, td))
			}
			None => Err(Error::BadProof),
		}
	}
}

/// Request for a header by hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderByHash(pub Field<H256>);

impl HeaderByHash {
	/// Check a response for the header.
	pub fn check_response(
		&self,
		cache: &Mutex<::cache::Cache>,
		start: &net_request::HashOrNumber,
		headers: &[encoded::Header]
	) -> Result<encoded::Header, Error> {
		let expected_hash = match (self.0, start) {
			(Field::Scalar(ref h), &net_request::HashOrNumber::Hash(ref h2)) => {
				if h != h2 { return Err(Error::WrongHash(*h, *h2)) }
				*h
			}
			(_, &net_request::HashOrNumber::Hash(h2)) => h2,
			_ => return Err(Error::HeaderByNumber),
		};

		let header = headers.get(0).ok_or(Error::Empty)?;
		let hash = header.hash();
		match hash == expected_hash {
			true => {
				cache.lock().insert_block_header(hash, header.clone());
				Ok(header.clone())
			}
			false => Err(Error::WrongHash(expected_hash, hash)),
		}
	}
}

/// Request for a transaction index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionIndex(pub Field<H256>);

impl TransactionIndex {
	/// Check a response for the transaction index.
	//
	// TODO: proper checking involves looking at canonicality of the
	// hash w.r.t. the current best block header.
	//
	// unlike all other forms of request, we don't know the header to check
	// until we make this request.
	//
	// This would require lookups in the database or perhaps CHT requests,
	// which aren't currently possible.
	//
	// Also, returning a result that is not locally canonical doesn't necessarily
	// indicate misbehavior, so the punishment scheme would need to be revised.
	pub fn check_response(
		&self,
		_cache: &Mutex<::cache::Cache>,
		res: &net_request::TransactionIndexResponse,
	) -> Result<net_request::TransactionIndexResponse, Error> {
		Ok(res.clone())
	}
}

/// Request for a block, with header for verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body(pub HeaderRef);

impl Body {
	/// Check a response for this block body.
	pub fn check_response(&self, cache: &Mutex<::cache::Cache>, body: &encoded::Body) -> Result<encoded::Block, Error> {
		// check the integrity of the the body against the header
		let header = self.0.as_ref()?;
		let tx_root = ::triehash::ordered_trie_root(body.rlp().at(0).iter().map(|r| r.as_raw().to_vec()));
		if tx_root != header.transactions_root() {
			return Err(Error::WrongTrieRoot(header.transactions_root(), tx_root));
		}

		let uncles_hash = keccak(body.rlp().at(1).as_raw());
		if uncles_hash != header.uncles_hash() {
			return Err(Error::WrongHash(header.uncles_hash(), uncles_hash));
		}

		// concatenate the header and the body.
		let mut stream = RlpStream::new_list(3);
		stream.append_raw(header.rlp().as_raw(), 1);
		stream.append_raw(body.rlp().at(0).as_raw(), 1);
		stream.append_raw(body.rlp().at(1).as_raw(), 1);

		cache.lock().insert_block_body(header.hash(), body.clone());

		Ok(encoded::Block::new(stream.out()))
	}
}

/// Request for a block's receipts with header for verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockReceipts(pub HeaderRef);

impl BlockReceipts {
	/// Check a response with receipts against the stored header.
	pub fn check_response(&self, cache: &Mutex<::cache::Cache>, receipts: &[Receipt]) -> Result<Vec<Receipt>, Error> {
		let receipts_root = self.0.as_ref()?.receipts_root();
		let found_root = ::triehash::ordered_trie_root(receipts.iter().map(|r| ::rlp::encode(r).into_vec()));

		match receipts_root == found_root {
			true => {
				cache.lock().insert_block_receipts(receipts_root, receipts.to_vec());
				Ok(receipts.to_vec())
			}
			false => Err(Error::WrongTrieRoot(receipts_root, found_root)),
		}
	}
}

/// Request for an account structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
	/// Header for verification.
	pub header: HeaderRef,
	/// Address requested.
	pub address: Address,
}

impl Account {
	/// Check a response with an account against the stored header.
	pub fn check_response(&self, _: &Mutex<::cache::Cache>, proof: &[Bytes]) -> Result<Option<BasicAccount>, Error> {
		let header = self.header.as_ref()?;
		let state_root = header.state_root();

		let mut db = MemoryDB::new();
		for node in proof { db.insert(&node[..]); }

		match TrieDB::new(&db, &state_root).and_then(|t| t.get(&keccak(&self.address)))? {
			Some(val) => {
				let rlp = UntrustedRlp::new(&val);
				Ok(Some(BasicAccount {
					nonce: rlp.val_at(0)?,
					balance: rlp.val_at(1)?,
					storage_root: rlp.val_at(2)?,
					code_hash: rlp.val_at(3)?,
				}))
			},
			None => Ok(None),
		}
	}
}

/// Request for account code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
	/// Header reference.
	pub header: HeaderRef,
	/// Account's code hash.
	pub code_hash: Field<H256>,
}

impl Code {
	/// Check a response with code against the code hash.
	pub fn check_response(
		&self,
		_: &Mutex<::cache::Cache>,
		code_hash: &H256,
		code: &[u8]
	) -> Result<Vec<u8>, Error> {
		let found_hash = keccak(code);
		if &found_hash == code_hash {
			Ok(code.to_vec())
		} else {
			Err(Error::WrongHash(*code_hash, found_hash))
		}
	}
}

/// Request for transaction execution, along with the parts necessary to verify the proof.
#[derive(Clone)]
pub struct TransactionProof {
	/// The transaction to request proof of.
	pub tx: SignedTransaction,
	/// Block header.
	pub header: HeaderRef,
	/// Transaction environment info.
	// TODO: it's not really possible to provide this if the header is unknown.
	pub env_info: EnvInfo,
	/// Consensus engine.
	pub engine: Arc<EthEngine>,
}

impl TransactionProof {
	/// Check the proof, returning the proved execution or indicate that the proof was bad.
	pub fn check_response(&self, _: &Mutex<::cache::Cache>, state_items: &[DBValue]) -> Result<super::ExecutionResult, Error> {
		let root = self.header.as_ref()?.state_root();

		let mut env_info = self.env_info.clone();
		env_info.gas_limit = self.tx.gas.clone();

		let proved_execution = state::check_proof(
			state_items,
			root,
			&self.tx,
			self.engine.machine(),
			&self.env_info,
		);

		match proved_execution {
			ProvedExecution::BadProof => Err(Error::BadProof),
			ProvedExecution::Failed(e) => Ok(Err(e)),
			ProvedExecution::Complete(e) => Ok(Ok(e)),
		}
	}
}

/// Request for epoch signal.
/// Provide engine and state-dependent proof checker.
#[derive(Clone)]
pub struct Signal {
	/// Block hash and number to fetch proof for.
	pub hash: H256,
	/// Consensus engine, used to check the proof.
	pub engine: Arc<EthEngine>,
	/// Special checker for the proof.
	pub proof_check: Arc<StateDependentProof<EthereumMachine>>,
}

impl Signal {
	/// Check the signal, returning the signal or indicate that it's bad.
	pub fn check_response(&self, _: &Mutex<::cache::Cache>, signal: &[u8]) -> Result<Vec<u8>, Error> {
		self.proof_check.check_proof(self.engine.machine(), signal)
			.map(|_| signal.to_owned())
			.map_err(|_| Error::BadProof)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bigint::hash::H256;
	use util::{MemoryDB, Address};
	use parking_lot::Mutex;
	use trie::{Trie, TrieMut, SecTrieDB, SecTrieDBMut};
	use trie::recorder::Recorder;
	use hash::keccak;

	use ethcore::client::{BlockChainClient, TestBlockChainClient, EachBlockWith};
	use ethcore::header::Header;
	use ethcore::encoded;
	use ethcore::receipt::{Receipt, TransactionOutcome};

	fn make_cache() -> ::cache::Cache {
		::cache::Cache::new(Default::default(), ::time::Duration::seconds(1))
	}

	#[test]
	fn no_invalid_header_by_number() {
		assert!(HeaderProof::new(0, Default::default()).is_none())
	}

	#[test]
	fn check_header_proof() {
		use ::cht;

		let test_client = TestBlockChainClient::new();
		test_client.add_blocks(10500, EachBlockWith::Nothing);

		let cht = {
			let fetcher = |id| {
				let hdr = test_client.block_header(id).unwrap();
				let td = test_client.block_total_difficulty(id).unwrap();
				Some(cht::BlockInfo {
					hash: hdr.hash(),
					parent_hash: hdr.parent_hash(),
					total_difficulty: td,
				})
			};

			cht::build(cht::block_to_cht_number(10_000).unwrap(), fetcher).unwrap()
		};

		let proof = cht.prove(10_000, 0).unwrap().unwrap();
		let req = HeaderProof::new(10_000, cht.root()).unwrap();

		let cache = Mutex::new(make_cache());
		assert!(req.check_response(&cache, &proof[..]).is_ok());
	}

	#[test]
	fn check_header_by_hash() {
		let mut header = Header::new();
		header.set_number(10_000);
		header.set_extra_data(b"test_header".to_vec());
		let hash = header.hash();
		let raw_header = encoded::Header::new(::rlp::encode(&header).into_vec());

		let cache = Mutex::new(make_cache());
		assert!(HeaderByHash(hash.into()).check_response(&cache, &hash.into(), &[raw_header]).is_ok())
	}

	#[test]
	fn check_body() {
		use rlp::RlpStream;

		let header = Header::new();
		let mut body_stream = RlpStream::new_list(2);
		body_stream.begin_list(0).begin_list(0);

		let req = Body(encoded::Header::new(::rlp::encode(&header).into_vec()).into());

		let cache = Mutex::new(make_cache());
		let response = encoded::Body::new(body_stream.drain().into_vec());
		assert!(req.check_response(&cache, &response).is_ok())
	}

	#[test]
	fn check_receipts() {
		let receipts = (0..5).map(|_| Receipt {
			outcome: TransactionOutcome::StateRoot(H256::random()),
			gas_used: 21_000u64.into(),
			log_bloom: Default::default(),
			logs: Vec::new(),
		}).collect::<Vec<_>>();

		let mut header = Header::new();
		let receipts_root = ::triehash::ordered_trie_root(
			receipts.iter().map(|x| ::rlp::encode(x).into_vec())
		);

		header.set_receipts_root(receipts_root);

		let req = BlockReceipts(encoded::Header::new(::rlp::encode(&header).into_vec()).into());

		let cache = Mutex::new(make_cache());
		assert!(req.check_response(&cache, &receipts).is_ok())
	}

	#[test]
	fn check_state_proof() {
		use rlp::RlpStream;

		let mut root = H256::default();
		let mut db = MemoryDB::new();
		let mut header = Header::new();
		header.set_number(123_456);
		header.set_extra_data(b"test_header".to_vec());

		let addr = Address::random();
		let rand_acc = || {
			let mut stream = RlpStream::new_list(4);
			stream.append(&2u64)
				.append(&100_000_000u64)
				.append(&H256::random())
				.append(&H256::random());

			stream.out()
		};
		{
			let mut trie = SecTrieDBMut::new(&mut db, &mut root);
			for _ in 0..100 {
				let address = Address::random();
				trie.insert(&*address, &rand_acc()).unwrap();
			}

			trie.insert(&*addr, &rand_acc()).unwrap();
		}

		let proof = {
			let trie = SecTrieDB::new(&db, &root).unwrap();
			let mut recorder = Recorder::new();

			trie.get_with(&*addr, &mut recorder).unwrap().unwrap();

			recorder.drain().into_iter().map(|r| r.data).collect::<Vec<_>>()
		};

		header.set_state_root(root.clone());

		let req = Account {
			header: encoded::Header::new(::rlp::encode(&header).into_vec()).into(),
			address: addr,
		};

		let cache = Mutex::new(make_cache());
		assert!(req.check_response(&cache, &proof[..]).is_ok());
	}

	#[test]
	fn check_code() {
		let code = vec![1u8; 256];
		let code_hash = keccak(&code);
		let header = Header::new();
		let req = Code {
			header: encoded::Header::new(::rlp::encode(&header).into_vec()).into(),
			code_hash: code_hash.into(),
		};

		let cache = Mutex::new(make_cache());
		assert!(req.check_response(&cache, &code_hash, &code).is_ok());
		assert!(req.check_response(&cache, &code_hash, &[]).is_err());
	}
}
