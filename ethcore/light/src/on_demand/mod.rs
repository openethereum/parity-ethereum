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

use ethcore::ids::BlockId;
use ethcore::block::Block;
use ethcore::header::Header;
use ethcore::receipt::Receipt;

use futures::Future;
use futures::sync::oneshot::{self, Sender, Receiver};
use network::PeerId;

use client::Client;
use net::{Handler, Status, Capabilities, Announcement, EventContext, BasicContext, ReqId};
use util::{Address, H256, RwLock};

struct Account;

// relevant peer info.
struct Peer {
	status: Status,
	capabilities: Capabilities,
}

// request info and where to send the result.
enum Request {
	HeaderByNumber(u64, H256, Sender<Header>), // num + CHT root
	HeaderByHash(H256, Sender<Header>),
	Block(Header, Sender<Block>),
	BlockReceipts(Header, Sender<Vec<Receipt>>),
	Account(Header, Address, Sender<Account>),
	Storage(Header, Address, H256, Sender<H256>),
}

/// On demand request service. See module docs for more details.
/// Accumulates info about all peers' capabilities and dispatches
/// requests to them accordingly.
pub struct OnDemand {
	peers: RwLock<HashMap<PeerId, Peer>>,
	pending_requests: RwLock<HashMap<ReqId, Request>>,
}

impl Handler for OnDemand {
	fn on_connect(&self, ctx: &EventContext, status: &Status, capabilities: &Capabilities) {
		self.peers.write().insert(ctx.peer(), Peer { status: status.clone(), capabilities: capabilities.clone() })
	}

	fn on_disconnect(&self, ctx: &EventContext, unfulfilled: &[ReqId]) {
		self.peers.write().remove(&ctx.peer());
	}
}

impl OnDemand {
	/// Request a header by block number and CHT root hash.
	pub fn header_by_number(&self, ctx: &BasicContext, num: u64, cht_root: H256) -> Receiver<Header> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_request(ctx, Request::HeaderByNumber(num, cht_root, sender));
		receiver
	}

	/// Request a header by hash. This is less accurate than by-number because we don't know
	/// where in the chain this header lies, and therefore can't find a peer who is supposed to have
	/// it as easily.
	pub fn header_by_hash(&self, ctx: &BasicContext, hash: H256) -> Receiver<Header> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_request(ctx, Request::HeaderByHash(hash, sender));
		receiver
	}

	/// Request a block, given its header. Block bodies are requestable by hash only,
	/// and the header is required anyway to verify and complete the block body
	/// -- this just doesn't obscure the network query.
	pub fn block(&self, ctx: &BasicContext, header: Header) -> Receiver<Block> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_request(ctx, Request::Block(header, sender));
		receiver
	}

	/// Request the receipts for a block. The header serves two purposes:
	/// provide the block hash to fetch receipts for, and for verification of the receipts root.
	pub fn block_receipts(&self, ctx: &BasicContext, header: Header) -> Receiver<Vec<Receipt>> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_request(ctx, Request::BlockReceipts(header, sender));
		receiver
	}

	/// Request an account by address and block header -- which gives a hash to query and a state root
	/// to verify against.
	pub fn account(&self, ctx: &BasicContext, header: Header, address: Address) -> Receiver<Account> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_request(ctx, Request::Account(header, address, sender));
		receiver
	}

	/// Request account storage value by block header, address, and key.
	pub fn storage(&self, ctx: &BasicContext, header: Header, address: Address, key: H256) -> Receiver<H256> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_request(ctx, Request::Storage(header, address, key, sender));
		receiver
	}

	// dispatch a request to a suitable peer.
	fn dispatch_request(&self, ctx: &BasicContext, request: Request) {
		unimplemented!()
	}
}
