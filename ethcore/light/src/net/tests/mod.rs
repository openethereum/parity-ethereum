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

//! Tests for the `LightProtocol` implementation.
//! These don't test of the higher level logic on top of

use ethcore::blockchain_info::BlockChainInfo;
use ethcore::client::{EachBlockWith, TestBlockChainClient};
use ethcore::ids::BlockId;
use ethcore::transaction::PendingTransaction;
use ethcore::encoded;
use network::{PeerId, NodeId};

use net::buffer_flow::FlowParams;
use net::context::IoContext;
use net::status::{Capabilities, Status, write_handshake};
use net::{encode_request, LightProtocol, Params, packet, Peer};
use provider::Provider;
use request::{self, Request, Headers};

use rlp::*;
use util::{Bytes, H256, U256};

use std::sync::Arc;

// expected result from a call.
#[derive(Debug, PartialEq, Eq)]
enum Expect {
	/// Expect to have message sent to peer.
	Send(PeerId, u8, Vec<u8>),
	/// Expect this response.
	Respond(u8, Vec<u8>),
	/// Expect a punishment (disconnect/disable)
	Punish(PeerId),
	/// Expect nothing.
	Nothing,
}

impl IoContext for Expect {
	fn send(&self, peer: PeerId, packet_id: u8, packet_body: Vec<u8>) {
		assert_eq!(self, &Expect::Send(peer, packet_id, packet_body));
	}

	fn respond(&self, packet_id: u8, packet_body: Vec<u8>) {
		assert_eq!(self, &Expect::Respond(packet_id, packet_body));
	}

	fn disconnect_peer(&self, peer: PeerId) {
		assert_eq!(self, &Expect::Punish(peer));
	}

	fn disable_peer(&self, peer: PeerId) {
		assert_eq!(self, &Expect::Punish(peer));
	}

	fn protocol_version(&self, _peer: PeerId) -> Option<u8> {
		Some(super::MAX_PROTOCOL_VERSION)
	}

	fn persistent_peer_id(&self, _peer: PeerId) -> Option<NodeId> {
		None
	}
}

// can't implement directly for Arc due to cross-crate orphan rules.
struct TestProvider(Arc<TestProviderInner>);

struct TestProviderInner {
	client: TestBlockChainClient,
}

impl Provider for TestProvider {
	fn chain_info(&self) -> BlockChainInfo {
		self.0.client.chain_info()
	}

	fn reorg_depth(&self, a: &H256, b: &H256) -> Option<u64> {
		self.0.client.reorg_depth(a, b)
	}

	fn earliest_state(&self) -> Option<u64> {
		None
	}

	fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		self.0.client.block_header(id)
	}

	fn block_body(&self, id: BlockId) -> Option<encoded::Body> {
		self.0.client.block_body(id)
	}

	fn block_receipts(&self, hash: &H256) -> Option<Bytes> {
		self.0.client.block_receipts(&hash)
	}

	fn state_proof(&self, req: request::StateProof) -> Vec<Bytes> {
		match req.key2 {
			Some(_) => vec![::util::sha3::SHA3_NULL_RLP.to_vec()],
			None => {
				// sort of a leaf node
				let mut stream = RlpStream::new_list(2);
				stream.append(&req.key1).append_empty_data();
				vec![stream.out()]
			}
		}
	}

	fn contract_code(&self, req: request::ContractCode) -> Bytes {
		req.account_key.iter().chain(req.account_key.iter()).cloned().collect()
	}

	fn header_proof(&self, _req: request::HeaderProof) -> Option<(encoded::Header, Vec<Bytes>)> {
		None
	}

	fn ready_transactions(&self) -> Vec<PendingTransaction> {
		self.0.client.ready_transactions()
	}
}

fn make_flow_params() -> FlowParams {
	FlowParams::new(5_000_000.into(), Default::default(), 100_000.into())
}

fn capabilities() -> Capabilities {
	Capabilities {
		serve_headers: true,
		serve_chain_since: Some(1),
		serve_state_since: Some(1),
		tx_relay: true,
	}
}

// helper for setting up the protocol handler and provider.
fn setup(flow_params: FlowParams, capabilities: Capabilities) -> (Arc<TestProviderInner>, LightProtocol) {
	let provider = Arc::new(TestProviderInner {
		client: TestBlockChainClient::new(),
	});

	let proto = LightProtocol::new(Arc::new(TestProvider(provider.clone())), Params {
		network_id: 2,
		flow_params: flow_params,
		capabilities: capabilities,
	});

	(provider, proto)
}

fn status(chain_info: BlockChainInfo) -> Status {
	Status {
		protocol_version: 1,
		network_id: 2,
		head_td: chain_info.total_difficulty,
		head_hash: chain_info.best_block_hash,
		head_num: chain_info.best_block_number,
		genesis_hash: chain_info.genesis_hash,
		last_head: None,
	}
}

#[test]
fn handshake_expected() {
	let flow_params = make_flow_params();
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let status = status(provider.client.chain_info());

	let packet_body = write_handshake(&status, &capabilities, Some(&flow_params));

	proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
}

#[test]
#[should_panic]
fn genesis_mismatch() {
	let flow_params = make_flow_params();
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let mut status = status(provider.client.chain_info());
	status.genesis_hash = H256::default();

	let packet_body = write_handshake(&status, &capabilities, Some(&flow_params));

	proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
}

#[test]
fn buffer_overflow() {
	let flow_params = make_flow_params();
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&status, &capabilities, Some(&flow_params));
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
	}

	{
		let my_status = write_handshake(&status, &capabilities, Some(&flow_params));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	// 1000 requests is far too many for the default flow params.
	let request = encode_request(&Request::Headers(Headers {
		start: 1.into(),
		max: 1000,
		skip: 0,
		reverse: false,
	}), 111);

	proto.handle_packet(&Expect::Punish(1), &1, packet::GET_BLOCK_HEADERS, &request);
}

// test the basic request types -- these just make sure that requests are parsed
// and sent to the provider correctly as well as testing response formatting.

#[test]
fn get_block_headers() {
	let flow_params = FlowParams::new(5_000_000.into(), Default::default(), 0.into());
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let cur_status = status(provider.client.chain_info());
	let my_status = write_handshake(&cur_status, &capabilities, Some(&flow_params));

	provider.client.add_blocks(100, EachBlockWith::Nothing);

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, Some(&flow_params));
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	let request = Headers {
		start: 1.into(),
		max: 10,
		skip: 0,
		reverse: false,
	};
	let req_id = 111;

	let request_body = encode_request(&Request::Headers(request.clone()), req_id);
	let response = {
		let headers: Vec<_> = (0..10).map(|i| provider.client.block_header(BlockId::Number(i + 1)).unwrap()).collect();
		assert_eq!(headers.len(), 10);

		let new_buf = *flow_params.limit() - flow_params.compute_cost(request::Kind::Headers, 10);

		let mut response_stream = RlpStream::new_list(3);

		response_stream.append(&req_id).append(&new_buf).begin_list(10);
		for header in headers {
			response_stream.append_raw(&header.into_inner(), 1);
		}

		response_stream.out()
	};

	let expected = Expect::Respond(packet::BLOCK_HEADERS, response);
	proto.handle_packet(&expected, &1, packet::GET_BLOCK_HEADERS, &request_body);
}

#[test]
fn get_block_bodies() {
	let flow_params = FlowParams::new(5_000_000.into(), Default::default(), 0.into());
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let cur_status = status(provider.client.chain_info());
	let my_status = write_handshake(&cur_status, &capabilities, Some(&flow_params));

	provider.client.add_blocks(100, EachBlockWith::Nothing);

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, Some(&flow_params));
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	let request = request::Bodies {
		block_hashes: (0..10).map(|i|
			provider.client.block_header(BlockId::Number(i)).unwrap().hash()
		).collect()
	};

	let req_id = 111;

	let request_body = encode_request(&Request::Bodies(request.clone()), req_id);
	let response = {
		let bodies: Vec<_> = (0..10).map(|i| provider.client.block_body(BlockId::Number(i + 1)).unwrap()).collect();
		assert_eq!(bodies.len(), 10);

		let new_buf = *flow_params.limit() - flow_params.compute_cost(request::Kind::Bodies, 10);

		let mut response_stream = RlpStream::new_list(3);

		response_stream.append(&req_id).append(&new_buf).begin_list(10);
		for body in bodies {
			response_stream.append_raw(&body.into_inner(), 1);
		}

		response_stream.out()
	};

	let expected = Expect::Respond(packet::BLOCK_BODIES, response);
	proto.handle_packet(&expected, &1, packet::GET_BLOCK_BODIES, &request_body);
}

#[test]
fn get_block_receipts() {
	let flow_params = FlowParams::new(5_000_000.into(), Default::default(), 0.into());
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let cur_status = status(provider.client.chain_info());
	let my_status = write_handshake(&cur_status, &capabilities, Some(&flow_params));

	provider.client.add_blocks(1000, EachBlockWith::Nothing);

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, Some(&flow_params));
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	// find the first 10 block hashes starting with `f` because receipts are only provided
	// by the test client in that case.
	let block_hashes: Vec<_> = (0..1000).map(|i|
		provider.client.block_header(BlockId::Number(i)).unwrap().hash()
	).filter(|hash| format!("{}", hash).starts_with("f")).take(10).collect();

	let request = request::Receipts {
		block_hashes: block_hashes.clone(),
	};

	let req_id = 111;

	let request_body = encode_request(&Request::Receipts(request.clone()), req_id);
	let response = {
		let receipts: Vec<_> = block_hashes.iter()
			.map(|hash| provider.client.block_receipts(hash).unwrap())
			.collect();

		let new_buf = *flow_params.limit() - flow_params.compute_cost(request::Kind::Receipts, receipts.len());

		let mut response_stream = RlpStream::new_list(3);

		response_stream.append(&req_id).append(&new_buf).begin_list(receipts.len());
		for block_receipts in receipts {
			response_stream.append_raw(&block_receipts, 1);
		}

		response_stream.out()
	};

	let expected = Expect::Respond(packet::RECEIPTS, response);
	proto.handle_packet(&expected, &1, packet::GET_RECEIPTS, &request_body);
}

#[test]
fn get_state_proofs() {
	let flow_params = FlowParams::new(5_000_000.into(), Default::default(), 0.into());
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, Some(&flow_params));
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body.clone()));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &packet_body);
	}

	let req_id = 112;
	let key1 = U256::from(11223344).into();
	let key2 = U256::from(99988887).into();

	let request = Request::StateProofs (request::StateProofs {
		requests: vec![
			request::StateProof { block: H256::default(), key1: key1, key2: None, from_level: 0 },
			request::StateProof { block: H256::default(), key1: key1, key2: Some(key2), from_level: 0},
		]
	});

	let request_body = encode_request(&request, req_id);
	let response = {
		let proofs = vec![
			{ let mut stream = RlpStream::new_list(2); stream.append(&key1).append_empty_data(); vec![stream.out()] },
			vec![::util::sha3::SHA3_NULL_RLP.to_vec()],
		];

		let new_buf = *flow_params.limit() - flow_params.compute_cost(request::Kind::StateProofs, 2);

		let mut response_stream = RlpStream::new_list(3);

		response_stream.append(&req_id).append(&new_buf).begin_list(2);
		for proof in proofs {
			response_stream.begin_list(proof.len());
			for node in proof {
				response_stream.append_raw(&node, 1);
			}
		}

		response_stream.out()
	};

	let expected = Expect::Respond(packet::PROOFS, response);
	proto.handle_packet(&expected, &1, packet::GET_PROOFS, &request_body);
}

#[test]
fn get_contract_code() {
	let flow_params = FlowParams::new(5_000_000.into(), Default::default(), 0.into());
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, Some(&flow_params));
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body.clone()));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &packet_body);
	}

	let req_id = 112;
	let key1 = U256::from(11223344).into();
	let key2 = U256::from(99988887).into();

	let request = Request::Codes (request::ContractCodes {
		code_requests: vec![
			request::ContractCode { block_hash: H256::default(), account_key: key1 },
			request::ContractCode { block_hash: H256::default(), account_key: key2 },
		],
	});

	let request_body = encode_request(&request, req_id);
	let response = {
		let codes: Vec<Vec<_>> = vec![
			key1.iter().chain(key1.iter()).cloned().collect(),
            key2.iter().chain(key2.iter()).cloned().collect(),
		];

		let new_buf = *flow_params.limit() - flow_params.compute_cost(request::Kind::Codes, 2);

		let mut response_stream = RlpStream::new_list(3);

		response_stream.append(&req_id).append(&new_buf).begin_list(2);
		for code in codes {
			response_stream.append(&code);
		}

		response_stream.out()
	};

	let expected = Expect::Respond(packet::CONTRACT_CODES, response);
	proto.handle_packet(&expected, &1, packet::GET_CONTRACT_CODES, &request_body);
}

#[test]
fn id_guard() {
	use super::request_set::RequestSet;
	use super::ReqId;

	let flow_params = FlowParams::new(5_000_000.into(), Default::default(), 0.into());
	let capabilities = capabilities();

	let (provider, proto) = setup(flow_params.clone(), capabilities.clone());

	let req_id_1 = ReqId(5143);
	let req_id_2 = ReqId(1111);
	let req = Request::Headers(request::Headers {
		start: 5u64.into(),
		max: 100,
		skip: 0,
		reverse: false,
	});

	let peer_id = 9876;

	let mut pending_requests = RequestSet::default();

	pending_requests.insert(req_id_1, req.clone(), ::time::SteadyTime::now());
	pending_requests.insert(req_id_2, req, ::time::SteadyTime::now());

	proto.peers.write().insert(peer_id, ::util::Mutex::new(Peer {
		local_buffer: flow_params.create_buffer(),
		status: status(provider.client.chain_info()),
		capabilities: capabilities.clone(),
		remote_flow: Some((flow_params.create_buffer(), flow_params)),
		sent_head: provider.client.chain_info().best_block_hash,
		last_update: ::time::SteadyTime::now(),
		pending_requests: pending_requests,
		failed_requests: Vec::new(),
	}));

	// first, supply wrong request type.
	{
		let mut stream = RlpStream::new_list(3);
		stream.append(&req_id_1.0);
		stream.append(&4_000_000usize);
		stream.begin_list(0);

		let packet = stream.out();
		assert!(proto.block_bodies(&peer_id, &Expect::Nothing, UntrustedRlp::new(&packet)).is_err());
	}

	// next, do an unexpected response.
	{
		let mut stream = RlpStream::new_list(3);
		stream.append(&10000usize);
		stream.append(&3_000_000usize);
		stream.begin_list(0);

		let packet = stream.out();
		assert!(proto.receipts(&peer_id, &Expect::Nothing, UntrustedRlp::new(&packet)).is_err());
	}

	// lastly, do a valid (but empty) response.
	{
		let mut stream = RlpStream::new_list(3);
		stream.append(&req_id_2.0);
		stream.append(&3_000_000usize);
		stream.begin_list(0);

		let packet = stream.out();
		assert!(proto.block_headers(&peer_id, &Expect::Nothing, UntrustedRlp::new(&packet)).is_ok());
	}

	let peers = proto.peers.read();
	if let Some(ref peer_info) = peers.get(&peer_id) {
		let peer_info = peer_info.lock();
		assert!(peer_info.pending_requests.collect_ids::<Vec<_>>().is_empty());
		assert_eq!(peer_info.failed_requests, &[req_id_1]);
	}
}
