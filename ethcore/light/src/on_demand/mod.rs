// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! On-demand chain requests over LES. This is a major building block for RPCs.
//! The request service is implemented using Futures. Higher level request handlers
//! will take the raw data received here and extract meaningful results from it.

use std::collections::HashMap;

use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::receipt::Receipt;

use futures::{Async, Poll, Future};
use futures::sync::oneshot;
use network::PeerId;

use net::{Handler, Status, Capabilities, Announcement, EventContext, BasicContext, ReqId};
use util::{Bytes, RwLock};
use types::les_request::{self as les_request, Request as LesRequest};

pub mod request;

/// Errors which can occur while trying to fulfill a request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
	/// Request was canceled.
	Canceled,
	/// No suitable peers available to serve the request.
	NoPeersAvailable,
	/// Invalid request.
	InvalidRequest,
	/// Request timed out.
	TimedOut,
}

impl From<oneshot::Canceled> for Error {
	fn from(_: oneshot::Canceled) -> Self {
		Error::Canceled
	}
}

/// Future which awaits a response to an on-demand request.
pub struct Response<T>(oneshot::Receiver<Result<T, Error>>);

impl<T> Future for Response<T> {
	type Item = T;
	type Error = Error;

	fn poll(&mut self) -> Poll<T, Error> {
		match self.0.poll().map_err(Into::into) {
			Ok(Async::Ready(val)) => val.map(Async::Ready),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(e) => Err(e),
		}
	}
}

type Sender<T> = oneshot::Sender<Result<T, Error>>;

// relevant peer info.
struct Peer {
	status: Status,
	capabilities: Capabilities,
}

// Attempted request info and sender to put received value.
enum Pending {
	HeaderByNumber(request::HeaderByNumber, Sender<encoded::Header>), // num + CHT root
	HeaderByHash(request::HeaderByHash, Sender<encoded::Header>),
	Block(request::Body, Sender<encoded::Block>),
	BlockReceipts(request::BlockReceipts, Sender<Vec<Receipt>>),
	Account(request::Account, Sender<BasicAccount>),
}

/// On demand request service. See module docs for more details.
/// Accumulates info about all peers' capabilities and dispatches
/// requests to them accordingly.
pub struct OnDemand {
	peers: RwLock<HashMap<PeerId, Peer>>,
	pending_requests: RwLock<HashMap<ReqId, Pending>>,
}

impl Default for OnDemand {
	fn default() -> Self {
		OnDemand {
			peers: RwLock::new(HashMap::new()),
			pending_requests: RwLock::new(HashMap::new()),
		}
	}
}

impl OnDemand {
	/// Request a header by block number and CHT root hash.
	pub fn header_by_number(&self, ctx: &BasicContext, req: request::HeaderByNumber) -> Response<encoded::Header> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_header_by_number(ctx, req, sender);
		Response(receiver)
	}

	// dispatch the request, completing the request if no peers available.
	fn dispatch_header_by_number(&self, ctx: &BasicContext, req: request::HeaderByNumber, sender: Sender<encoded::Header>) {
		let num = req.num;
		let cht_num = match ::cht::block_to_cht_number(req.num) {
			Some(cht_num) => cht_num,
			None => {
				warn!(target: "on_demand", "Attempted to dispatch invalid header proof: req.num == 0");
				sender.complete(Err(Error::InvalidRequest));
				return;
			}
		};

		let les_req = LesRequest::HeaderProofs(les_request::HeaderProofs {
			requests: vec![les_request::HeaderProof {
				cht_number: cht_num,
				block_number: req.num,
				from_level: 0,
			}],
		});

		// we're looking for a peer with serveHeaders who's far enough along in the
		// chain.
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_headers && peer.status.head_num >= num {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							Pending::HeaderByNumber(req, sender)
						);
						return
					},
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		// TODO: retrying
		trace!(target: "on_demand", "No suitable peer for request");
		sender.complete(Err(Error::NoPeersAvailable));
	}

	/// Request a header by hash. This is less accurate than by-number because we don't know
	/// where in the chain this header lies, and therefore can't find a peer who is supposed to have
	/// it as easily.
	pub fn header_by_hash(&self, ctx: &BasicContext, req: request::HeaderByHash) -> Response<encoded::Header> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_header_by_hash(ctx, req, sender);
		Response(receiver)
	}

	fn dispatch_header_by_hash(&self, ctx: &BasicContext, req: request::HeaderByHash, sender: Sender<encoded::Header>) {
		let les_req = LesRequest::Headers(les_request::Headers {
			start: req.0.into(),
			max: 1,
			skip: 0,
			reverse: false,
		});

		// all we've got is a hash, so we'll just guess at peers who might have
		// it randomly.
		let mut potential_peers = self.peers.read().iter()
			.filter(|&(_, peer)| peer.capabilities.serve_headers)
			.map(|(id, _)| *id)
			.collect::<Vec<_>>();

		let mut rng = ::rand::thread_rng();

		::rand::Rng::shuffle(&mut rng, &mut potential_peers);

		for id in potential_peers {
			match ctx.request_from(id, les_req.clone()) {
				Ok(req_id) => {
					trace!(target: "on_demand", "Assigning request to peer {}", id);
					self.pending_requests.write().insert(
						req_id,
						Pending::HeaderByHash(req, sender),
					);
					return
				}
				Err(e) =>
					trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
			}
		}

		// TODO: retrying
		trace!(target: "on_demand", "No suitable peer for request");
		sender.complete(Err(Error::NoPeersAvailable));
	}

	/// Request a block, given its header. Block bodies are requestable by hash only,
	/// and the header is required anyway to verify and complete the block body
	/// -- this just doesn't obscure the network query.
	pub fn block(&self, ctx: &BasicContext, req: request::Body) -> Response<encoded::Block> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_block(ctx, req, sender);
		Response(receiver)
	}

	fn dispatch_block(&self, ctx: &BasicContext, req: request::Body, sender: Sender<encoded::Block>) {
		let num = req.header.number();
		let les_req = LesRequest::Bodies(les_request::Bodies {
			block_hashes: vec![req.hash],
		});

		// we're looking for a peer with serveChainSince(num)
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_chain_since.as_ref().map_or(false, |x| *x >= num) {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							Pending::Block(req, sender)
						);
						return
					}
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		// TODO: retrying
		trace!(target: "on_demand", "No suitable peer for request");
		sender.complete(Err(Error::NoPeersAvailable));
	}

	/// Request the receipts for a block. The header serves two purposes:
	/// provide the block hash to fetch receipts for, and for verification of the receipts root.
	pub fn block_receipts(&self, ctx: &BasicContext, req: request::BlockReceipts) -> Response<Vec<Receipt>> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_block_receipts(ctx, req, sender);
		Response(receiver)
	}

	fn dispatch_block_receipts(&self, ctx: &BasicContext, req: request::BlockReceipts, sender: Sender<Vec<Receipt>>) {
		let num = req.0.number();
		let les_req = LesRequest::Receipts(les_request::Receipts {
			block_hashes: vec![req.0.hash()],
		});

		// we're looking for a peer with serveChainSince(num)
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_chain_since.as_ref().map_or(false, |x| *x >= num) {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							Pending::BlockReceipts(req, sender)
						);
						return
					}
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		// TODO: retrying
		trace!(target: "on_demand", "No suitable peer for request");
		sender.complete(Err(Error::NoPeersAvailable));
	}

	/// Request an account by address and block header -- which gives a hash to query and a state root
	/// to verify against.
	pub fn account(&self, ctx: &BasicContext, req: request::Account) -> Response<BasicAccount> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_account(ctx, req, sender);
		Response(receiver)
	}

	fn dispatch_account(&self, ctx: &BasicContext, req: request::Account, sender: Sender<BasicAccount>) {
		let num = req.header.number();
		let les_req = LesRequest::StateProofs(les_request::StateProofs {
			requests: vec![les_request::StateProof {
				block: req.header.hash(),
				key1: ::util::Hashable::sha3(&req.address),
				key2: None,
				from_level: 0,
			}],
		});

		// we're looking for a peer with serveStateSince(num)
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_state_since.as_ref().map_or(false, |x| *x >= num) {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							Pending::Account(req, sender)
						);
						return
					}
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		// TODO: retrying
		trace!(target: "on_demand", "No suitable peer for request");
		sender.complete(Err(Error::NoPeersAvailable));
	}
}

impl Handler for OnDemand {
	fn on_connect(&self, ctx: &EventContext, status: &Status, capabilities: &Capabilities) {
		self.peers.write().insert(ctx.peer(), Peer { status: status.clone(), capabilities: capabilities.clone() });
	}

	fn on_disconnect(&self, ctx: &EventContext, unfulfilled: &[ReqId]) {
		self.peers.write().remove(&ctx.peer());
		let ctx = ctx.as_basic();

		for unfulfilled in unfulfilled {
			if let Some(pending) = self.pending_requests.write().remove(unfulfilled) {
				trace!(target: "on_demand", "Attempting to reassign dropped request");
				match pending {
					Pending::HeaderByNumber(req, sender)
						=> self.dispatch_header_by_number(ctx, req, sender),
					Pending::HeaderByHash(req, sender)
						=> self.dispatch_header_by_hash(ctx, req, sender),
					Pending::Block(req, sender)
						=> self.dispatch_block(ctx, req, sender),
					Pending::BlockReceipts(req, sender)
						=> self.dispatch_block_receipts(ctx, req, sender),
					Pending::Account(req, sender)
						=> self.dispatch_account(ctx, req, sender),
				}
			}
		}
	}

	fn on_announcement(&self, ctx: &EventContext, announcement: &Announcement) {
		let mut peers = self.peers.write();
		if let Some(ref mut peer) = peers.get_mut(&ctx.peer()) {
			peer.status.update_from(&announcement);
			peer.capabilities.update_from(&announcement);
		}
	}

	fn on_header_proofs(&self, ctx: &EventContext, req_id: ReqId, proofs: &[(Bytes, Vec<Bytes>)]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::HeaderByNumber(req, sender) => {
				if let Some(&(ref header, ref proof)) = proofs.get(0) {
					match req.check_response(header, proof) {
						Ok(header) => {
							sender.complete(Ok(header));
							return
						}
						Err(e) => {
							warn!("Error handling response for header request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_header_by_number(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only header by number request fetches header proofs; qed"),
		}
	}

	fn on_block_headers(&self, ctx: &EventContext, req_id: ReqId, headers: &[Bytes]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::HeaderByHash(req, sender) => {
				if let Some(ref header) = headers.get(0) {
					match req.check_response(header) {
						Ok(header) => {
							sender.complete(Ok(header));
							return
						}
						Err(e) => {
							warn!("Error handling response for header request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_header_by_hash(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only header by hash request fetches headers; qed"),
		}
	}

	fn on_block_bodies(&self, ctx: &EventContext, req_id: ReqId, bodies: &[Bytes]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::Block(req, sender) => {
				if let Some(ref block) = bodies.get(0) {
					match req.check_response(block) {
						Ok(block) => {
							sender.complete(Ok(block));
							return
						}
						Err(e) => {
							warn!("Error handling response for block request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_block(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only block request fetches bodies; qed"),
		}
	}

	fn on_receipts(&self, ctx: &EventContext, req_id: ReqId, receipts: &[Vec<Receipt>]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::BlockReceipts(req, sender) => {
				if let Some(ref receipts) = receipts.get(0) {
					match req.check_response(receipts) {
						Ok(receipts) => {
							sender.complete(Ok(receipts));
							return
						}
						Err(e) => {
							warn!("Error handling response for receipts request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_block_receipts(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only receipts request fetches receipts; qed"),
		}
	}

	fn on_state_proofs(&self, ctx: &EventContext, req_id: ReqId, proofs: &[Vec<Bytes>]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::Account(req, sender) => {
				if let Some(ref proof) = proofs.get(0) {
					match req.check_response(proof) {
						Ok(proof) => {
							sender.complete(Ok(proof));
							return
						}
						Err(e) => {
							warn!("Error handling response for state request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_account(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only account request fetches state proof; qed"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use net::{Announcement, BasicContext, ReqId, Error as LesError};
	use request::{Request as LesRequest, Kind as LesRequestKind};
	use network::{PeerId, NodeId};
	use futures::Future;
	use util::H256;

	struct FakeContext;

	impl BasicContext for FakeContext {
		fn persistent_peer_id(&self, _: PeerId) -> Option<NodeId> { None }
		fn request_from(&self, _: PeerId, _: LesRequest) -> Result<ReqId, LesError> {
			unimplemented!()
		}
		fn make_announcement(&self, _: Announcement) { }
		fn max_requests(&self, _: PeerId, _: LesRequestKind) -> usize { 0 }
		fn disconnect_peer(&self, _: PeerId) { }
		fn disable_peer(&self, _: PeerId) { }
	}

	#[test]
	fn no_peers() {
		let on_demand = OnDemand::default();
		let result = on_demand.header_by_hash(&FakeContext, request::HeaderByHash(H256::default()));

		assert_eq!(result.wait().unwrap_err(), Error::NoPeersAvailable);
	}
}
