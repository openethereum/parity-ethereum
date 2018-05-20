// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Tests for the on-demand service.

use cache::Cache;
use ethcore::header::Header;
use futures::Future;
use network::{PeerId, NodeId};
use net::*;
use ethereum_types::H256;
use parking_lot::Mutex;
use std::time::Duration;
use ::request::{self as basic_request, Response};

use std::sync::Arc;

use super::{request, OnDemand, Peer, HeaderRef};

// useful contexts to give the service.
enum Context {
	NoOp,
	WithPeer(PeerId),
	RequestFrom(PeerId, ReqId),
	Punish(PeerId),
}

impl EventContext for Context {
	fn peer(&self) -> PeerId {
		match *self {
			Context::WithPeer(id)
			| Context::RequestFrom(id, _)
			| Context::Punish(id) => id,
			_ => panic!("didn't expect to have peer queried."),
		}
	}

	fn as_basic(&self) -> &BasicContext { self }
}

impl BasicContext for Context {
	/// Returns the relevant's peer persistent Id (aka NodeId).
	fn persistent_peer_id(&self, _: PeerId) -> Option<NodeId> {
		panic!("didn't expect to provide persistent ID")
	}

	fn request_from(&self, peer_id: PeerId, _: ::request::NetworkRequests) -> Result<ReqId, Error> {
		match *self {
			Context::RequestFrom(id, req_id) => if peer_id == id { Ok(req_id) } else { Err(Error::NoCredits) },
			_ => panic!("didn't expect to have requests dispatched."),
		}
	}

	fn make_announcement(&self, _: Announcement) {
		panic!("didn't expect to make announcement")
	}

	fn disconnect_peer(&self, id: PeerId) {
		self.disable_peer(id)
	}

	fn disable_peer(&self, peer_id: PeerId) {
		match *self {
			Context::Punish(id) if id == peer_id => {},
			_ => panic!("Unexpectedly punished peer."),
		}
	}
}

// test harness.
struct Harness {
	service: OnDemand,
}

impl Harness {
	fn create() -> Self {
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(60))));
		Harness {
			service: OnDemand::new_test(cache),
		}
	}

	fn inject_peer(&self, id: PeerId, peer: Peer) {
		self.service.peers.write().insert(id, peer);
	}
}

fn dummy_status() -> Status {
	Status {
		protocol_version: 1,
		network_id: 999,
		head_td: 1.into(),
		head_hash: H256::default(),
		head_num: 1359,
		genesis_hash: H256::default(),
		last_head: None,
	}
}

fn dummy_capabilities() -> Capabilities {
	Capabilities {
		serve_headers: true,
		serve_chain_since: Some(1),
		serve_state_since: Some(1),
		tx_relay: true,
	}
}

#[test]
fn detects_hangup() {
	let on_demand = Harness::create().service;
	let result = on_demand.request_raw(
		&Context::NoOp,
		vec![request::HeaderByHash(H256::default().into()).into()],
	);

	assert_eq!(on_demand.pending.read().len(), 1);
	drop(result);

	on_demand.dispatch_pending(&Context::NoOp);
	assert!(on_demand.pending.read().is_empty());
}

#[test]
fn single_request() {
	let harness = Harness::create();

	let peer_id = 10101;
	let req_id = ReqId(14426);

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	let header = Header::default();
	let encoded = header.encoded();

	let recv = harness.service.request_raw(
		&Context::NoOp,
		vec![request::HeaderByHash(header.hash().into()).into()]
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_id));

	assert_eq!(harness.service.pending.read().len(), 0);

	harness.service.on_responses(
		&Context::WithPeer(peer_id),
		req_id,
		&[Response::Headers(basic_request::HeadersResponse { headers: vec![encoded] })]
	);

	assert!(recv.wait().is_ok());
}

#[test]
fn no_capabilities() {
	let harness = Harness::create();

	let peer_id = 10101;

	let mut capabilities = dummy_capabilities();
	capabilities.serve_headers = false;

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: capabilities,
	});

	let _recv = harness.service.request_raw(
		&Context::NoOp,
		vec![request::HeaderByHash(H256::default().into()).into()]
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::NoOp);

	assert_eq!(harness.service.pending.read().len(), 1);
}

#[test]
fn reassign() {
	let harness = Harness::create();

	let peer_ids = (10101, 12345);
	let req_ids = (ReqId(14426), ReqId(555));

	harness.inject_peer(peer_ids.0, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	let header = Header::default();
	let encoded = header.encoded();

	let recv = harness.service.request_raw(
		&Context::NoOp,
		vec![request::HeaderByHash(header.hash().into()).into()]
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_ids.0, req_ids.0));
	assert_eq!(harness.service.pending.read().len(), 0);

	harness.service.on_disconnect(&Context::WithPeer(peer_ids.0), &[req_ids.0]);
	assert_eq!(harness.service.pending.read().len(), 1);

	harness.inject_peer(peer_ids.1, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	harness.service.dispatch_pending(&Context::RequestFrom(peer_ids.1, req_ids.1));
	assert_eq!(harness.service.pending.read().len(), 0);

	harness.service.on_responses(
		&Context::WithPeer(peer_ids.1),
		req_ids.1,
		&[Response::Headers(basic_request::HeadersResponse { headers: vec![encoded] })]
	);

	assert!(recv.wait().is_ok());
}

#[test]
fn partial_response() {
	let harness = Harness::create();

	let peer_id = 111;
	let req_ids = (ReqId(14426), ReqId(555));

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	let make = |num| {
		let mut hdr = Header::default();
		hdr.set_number(num);

		let encoded = hdr.encoded();
		(hdr, encoded)
	};

	let (header1, encoded1) = make(5);
	let (header2, encoded2) = make(23452);

	// request two headers.
	let recv = harness.service.request_raw(
		&Context::NoOp,
		vec![
			request::HeaderByHash(header1.hash().into()).into(),
			request::HeaderByHash(header2.hash().into()).into(),
		],
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_ids.0));
	assert_eq!(harness.service.pending.read().len(), 0);

	// supply only the first one.
	harness.service.on_responses(
		&Context::WithPeer(peer_id),
		req_ids.0,
		&[Response::Headers(basic_request::HeadersResponse { headers: vec![encoded1] })]
	);

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_ids.1));
	assert_eq!(harness.service.pending.read().len(), 0);

	// supply the next one.
	harness.service.on_responses(
		&Context::WithPeer(peer_id),
		req_ids.1,
		&[Response::Headers(basic_request::HeadersResponse { headers: vec![encoded2] })]
	);

	assert!(recv.wait().is_ok());
}

#[test]
fn part_bad_part_good() {
	let harness = Harness::create();

	let peer_id = 111;
	let req_ids = (ReqId(14426), ReqId(555));

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	let make = |num| {
		let mut hdr = Header::default();
		hdr.set_number(num);

		let encoded = hdr.encoded();
		(hdr, encoded)
	};

	let (header1, encoded1) = make(5);
	let (header2, encoded2) = make(23452);

	// request two headers.
	let recv = harness.service.request_raw(
		&Context::NoOp,
		vec![
			request::HeaderByHash(header1.hash().into()).into(),
			request::HeaderByHash(header2.hash().into()).into(),
		],
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_ids.0));
	assert_eq!(harness.service.pending.read().len(), 0);

	// supply only the first one, but followed by the wrong kind of response.
	// the first header should be processed.
	harness.service.on_responses(
		&Context::Punish(peer_id),
		req_ids.0,
		&[
			Response::Headers(basic_request::HeadersResponse { headers: vec![encoded1] }),
			Response::Receipts(basic_request::ReceiptsResponse { receipts: vec![] } ),
		]
	);

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_ids.1));
	assert_eq!(harness.service.pending.read().len(), 0);

	// supply the next one.
	harness.service.on_responses(
		&Context::WithPeer(peer_id),
		req_ids.1,
		&[Response::Headers(basic_request::HeadersResponse { headers: vec![encoded2] })]
	);

	assert!(recv.wait().is_ok());
}

#[test]
fn wrong_kind() {
	let harness = Harness::create();

	let peer_id = 10101;
	let req_id = ReqId(14426);

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	let _recv = harness.service.request_raw(
		&Context::NoOp,
		vec![request::HeaderByHash(H256::default().into()).into()]
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_id));

	assert_eq!(harness.service.pending.read().len(), 0);

	harness.service.on_responses(
		&Context::Punish(peer_id),
		req_id,
		&[Response::Receipts(basic_request::ReceiptsResponse { receipts: vec![] })]
	);

	assert_eq!(harness.service.pending.read().len(), 1);
}

#[test]
fn back_references() {
	let harness = Harness::create();

	let peer_id = 10101;
	let req_id = ReqId(14426);

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	let header = Header::default();
	let encoded = header.encoded();

	let recv = harness.service.request_raw(
		&Context::NoOp,
		vec![
			request::HeaderByHash(header.hash().into()).into(),
			request::BlockReceipts(HeaderRef::Unresolved(0, header.hash().into())).into(),
		]
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_id));

	assert_eq!(harness.service.pending.read().len(), 0);

	harness.service.on_responses(
		&Context::WithPeer(peer_id),
		req_id,
		&[
			Response::Headers(basic_request::HeadersResponse { headers: vec![encoded] }),
			Response::Receipts(basic_request::ReceiptsResponse { receipts: vec![] }),
		]
	);

	assert!(recv.wait().is_ok());
}

#[test]
#[should_panic]
fn bad_back_reference() {
	let harness = Harness::create();

	let header = Header::default();

	let _ = harness.service.request_raw(
		&Context::NoOp,
		vec![
			request::HeaderByHash(header.hash().into()).into(),
			request::BlockReceipts(HeaderRef::Unresolved(1, header.hash().into())).into(),
		]
	).unwrap();
}

#[test]
fn fill_from_cache() {
	let harness = Harness::create();

	let peer_id = 10101;
	let req_id = ReqId(14426);

	harness.inject_peer(peer_id, Peer {
		status: dummy_status(),
		capabilities: dummy_capabilities(),
	});

	let header = Header::default();
	let encoded = header.encoded();

	let recv = harness.service.request_raw(
		&Context::NoOp,
		vec![
			request::HeaderByHash(header.hash().into()).into(),
			request::BlockReceipts(HeaderRef::Unresolved(0, header.hash().into())).into(),
		]
	).unwrap();

	assert_eq!(harness.service.pending.read().len(), 1);

	harness.service.dispatch_pending(&Context::RequestFrom(peer_id, req_id));

	assert_eq!(harness.service.pending.read().len(), 0);

	harness.service.on_responses(
		&Context::WithPeer(peer_id),
		req_id,
		&[
			Response::Headers(basic_request::HeadersResponse { headers: vec![encoded] }),
		]
	);

	assert!(recv.wait().is_ok());
}
