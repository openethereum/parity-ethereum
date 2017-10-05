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
use ethcore::transaction::{Action, PendingTransaction};
use ethcore::encoded;
use network::{PeerId, NodeId};

use net::context::IoContext;
use net::status::{Capabilities, Status};
use net::{LightProtocol, Params, packet, Peer};
use provider::Provider;
use request;
use request::*;

use rlp::*;
use bigint::prelude::U256;
use bigint::hash::H256;
use util::Address;

use std::sync::Arc;

// helper for encoding a single request into a packet.
// panics on bad backreference.
fn encode_single(request: Request) -> NetworkRequests {
	let mut builder = Builder::default();
	builder.push(request).unwrap();
	builder.build()
}

// helper for making a packet out of `Requests`.
fn make_packet(req_id: usize, requests: &NetworkRequests) -> Vec<u8> {
	let mut stream = RlpStream::new_list(2);
	stream.append(&req_id).append_list(&requests.requests());
	stream.out()
}

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

	fn transaction_index(&self, req: request::CompleteTransactionIndexRequest)
		-> Option<request::TransactionIndexResponse>
	{
		Some(request::TransactionIndexResponse {
			num: 100,
			hash: req.hash,
			index: 55,
		})
	}

	fn block_body(&self, req: request::CompleteBodyRequest) -> Option<request::BodyResponse> {
		self.0.client.block_body(req)
	}

	fn block_receipts(&self, req: request::CompleteReceiptsRequest) -> Option<request::ReceiptsResponse> {
		self.0.client.block_receipts(req)
	}

	fn account_proof(&self, req: request::CompleteAccountRequest) -> Option<request::AccountResponse> {
		// sort of a leaf node
		let mut stream = RlpStream::new_list(2);
		stream.append(&req.address_hash).append_empty_data();
		Some(AccountResponse {
			proof: vec![stream.out()],
			balance: 10.into(),
			nonce: 100.into(),
			code_hash: Default::default(),
			storage_root: Default::default(),
		})
	}

	fn storage_proof(&self, req: request::CompleteStorageRequest) -> Option<request::StorageResponse> {
		Some(StorageResponse {
			proof: vec![::rlp::encode(&req.key_hash).into_vec()],
			value: req.key_hash | req.address_hash,
		})
	}

	fn contract_code(&self, req: request::CompleteCodeRequest) -> Option<request::CodeResponse> {
		Some(CodeResponse {
			code: req.block_hash.iter().chain(req.code_hash.iter()).cloned().collect(),
		})
	}

	fn header_proof(&self, _req: request::CompleteHeaderProofRequest) -> Option<request::HeaderProofResponse> {
		None
	}

	fn transaction_proof(&self, _req: request::CompleteExecutionRequest) -> Option<request::ExecutionResponse> {
		None
	}

	fn epoch_signal(&self, _req: request::CompleteSignalRequest) -> Option<request::SignalResponse> {
		Some(request::SignalResponse {
			signal: vec![1, 2, 3, 4],
		})
	}

	fn ready_transactions(&self) -> Vec<PendingTransaction> {
		self.0.client.ready_transactions()
	}
}

fn capabilities() -> Capabilities {
	Capabilities {
		serve_headers: true,
		serve_chain_since: Some(1),
		serve_state_since: Some(1),
		tx_relay: true,
	}
}

fn write_handshake(status: &Status, capabilities: &Capabilities, proto: &LightProtocol) -> Vec<u8> {
	let flow_params = proto.flow_params.read().clone();
	::net::status::write_handshake(status, capabilities, Some(&*flow_params))
}

// helper for setting up the protocol handler and provider.
fn setup(capabilities: Capabilities) -> (Arc<TestProviderInner>, LightProtocol) {
	let provider = Arc::new(TestProviderInner {
		client: TestBlockChainClient::new(),
	});

	let proto = LightProtocol::new(Arc::new(TestProvider(provider.clone())), Params {
		network_id: 2,
		config: Default::default(),
		capabilities: capabilities,
		sample_store: None,
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
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());

	let status = status(provider.client.chain_info());

	let packet_body = write_handshake(&status, &capabilities, &proto);

	proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
}

#[test]
#[should_panic]
fn genesis_mismatch() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());

	let mut status = status(provider.client.chain_info());
	status.genesis_hash = H256::default();

	let packet_body = write_handshake(&status, &capabilities, &proto);

	proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
}

#[test]
fn credit_overflow() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());

	let status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
	}

	{
		let my_status = write_handshake(&status, &capabilities, &proto);
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	// 1 billion requests is far too many for the default flow params.
	let requests = encode_single(Request::Headers(IncompleteHeadersRequest {
		start: HashOrNumber::Number(1).into(),
		max: 1_000_000_000,
		skip: 0,
		reverse: false,
	}));
	let request = make_packet(111, &requests);

	proto.handle_packet(&Expect::Punish(1), &1, packet::REQUEST, &request);
}

// test the basic request types -- these just make sure that requests are parsed
// and sent to the provider correctly as well as testing response formatting.

#[test]
fn get_block_headers() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let cur_status = status(provider.client.chain_info());
	let my_status = write_handshake(&cur_status, &capabilities, &proto);

	provider.client.add_blocks(100, EachBlockWith::Nothing);

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	let request = Request::Headers(IncompleteHeadersRequest {
		start: HashOrNumber::Number(1).into(),
		max: 10,
		skip: 0,
		reverse: false,
	});

	let req_id = 111;

	let requests = encode_single(request.clone());
	let request_body = make_packet(req_id, &requests);

	let response = {
		let headers: Vec<_> = (0..10).map(|i| provider.client.block_header(BlockId::Number(i + 1)).unwrap()).collect();
		assert_eq!(headers.len(), 10);

		let new_creds = *flow_params.limit() - flow_params.compute_cost_multi(requests.requests()).unwrap();

		let response = vec![Response::Headers(HeadersResponse {
			headers: headers,
		})];

		let mut stream = RlpStream::new_list(3);
		stream.append(&req_id).append(&new_creds).append_list(&response);

		stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}

#[test]
fn get_block_bodies() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let cur_status = status(provider.client.chain_info());
	let my_status = write_handshake(&cur_status, &capabilities, &proto);

	provider.client.add_blocks(100, EachBlockWith::Nothing);

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	let mut builder = Builder::default();
	let mut bodies = Vec::new();

	for i in 0..10 {
		let hash = provider.client.block_header(BlockId::Number(i)).unwrap().hash();
		builder.push(Request::Body(IncompleteBodyRequest {
			hash: hash.into(),
		})).unwrap();
		bodies.push(Response::Body(provider.client.block_body(CompleteBodyRequest {
			hash: hash,
		}).unwrap()));
	}
	let req_id = 111;
	let requests = builder.build();
	let request_body = make_packet(req_id, &requests);

	let response = {
		let new_creds = *flow_params.limit() - flow_params.compute_cost_multi(requests.requests()).unwrap();

		let mut response_stream = RlpStream::new_list(3);
		response_stream.append(&req_id).append(&new_creds).append_list(&bodies);
		response_stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}

#[test]
fn get_block_receipts() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let cur_status = status(provider.client.chain_info());
	let my_status = write_handshake(&cur_status, &capabilities, &proto);

	provider.client.add_blocks(1000, EachBlockWith::Nothing);

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &my_status);
	}

	// find the first 10 block hashes starting with `f` because receipts are only provided
	// by the test client in that case.
	let block_hashes: Vec<H256> = (0..1000)
		.map(|i| provider.client.block_header(BlockId::Number(i)).unwrap().hash())
		.filter(|hash| format!("{}", hash).starts_with("f"))
		.take(10)
		.collect();

	let mut builder = Builder::default();
	let mut receipts = Vec::new();
	for hash in block_hashes.iter().cloned() {
		builder.push(Request::Receipts(IncompleteReceiptsRequest { hash: hash.into() })).unwrap();
		receipts.push(Response::Receipts(provider.client.block_receipts(CompleteReceiptsRequest {
			hash: hash
		}).unwrap()));
	}

	let req_id = 111;
	let requests = builder.build();
	let request_body = make_packet(req_id, &requests);

	let response = {
		assert_eq!(receipts.len(), 10);

		let new_creds = *flow_params.limit() - flow_params.compute_cost_multi(requests.requests()).unwrap();

		let mut response_stream = RlpStream::new_list(3);
		response_stream.append(&req_id).append(&new_creds).append_list(&receipts);
		response_stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}

#[test]
fn get_state_proofs() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let provider = TestProvider(provider);

	let cur_status = status(provider.0.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body.clone()));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &packet_body);
	}

	let req_id = 112;
	let key1: H256 = U256::from(11223344).into();
	let key2: H256 = U256::from(99988887).into();

	let mut builder = Builder::default();
	builder.push(Request::Account(IncompleteAccountRequest {
		block_hash: H256::default().into(),
		address_hash: key1.into(),
	})).unwrap();
	builder.push(Request::Storage(IncompleteStorageRequest {
		block_hash: H256::default().into(),
		address_hash: key1.into(),
		key_hash: key2.into(),
	})).unwrap();

	let requests = builder.build();

	let request_body = make_packet(req_id, &requests);
	let response = {
		let responses = vec![
			Response::Account(provider.account_proof(CompleteAccountRequest {
				block_hash: H256::default(),
				address_hash: key1,
			}).unwrap()),
			Response::Storage(provider.storage_proof(CompleteStorageRequest {
				block_hash: H256::default(),
				address_hash: key1,
				key_hash: key2,
			}).unwrap()),
		];

		let new_creds = *flow_params.limit() - flow_params.compute_cost_multi(requests.requests()).unwrap();

		let mut response_stream = RlpStream::new_list(3);
		response_stream.append(&req_id).append(&new_creds).append_list(&responses);
		response_stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}

#[test]
fn get_contract_code() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body.clone()));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &packet_body);
	}

	let req_id = 112;
	let key1: H256 = U256::from(11223344).into();
	let key2: H256 = U256::from(99988887).into();

	let request = Request::Code(IncompleteCodeRequest {
		block_hash: key1.into(),
		code_hash: key2.into(),
	});

	let requests = encode_single(request.clone());
	let request_body = make_packet(req_id, &requests);
	let response = {
		let response = vec![Response::Code(CodeResponse {
			code: key1.iter().chain(key2.iter()).cloned().collect(),
		})];

		let new_creds = *flow_params.limit() - flow_params.compute_cost_multi(requests.requests()).unwrap();

		let mut response_stream = RlpStream::new_list(3);

		response_stream.append(&req_id).append(&new_creds).append_list(&response);
		response_stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}

#[test]
fn epoch_signal() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body.clone()));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &packet_body);
	}

	let req_id = 112;
	let request = Request::Signal(request::IncompleteSignalRequest {
		block_hash: H256([1; 32]).into(),
	});

	let requests = encode_single(request.clone());
	let request_body = make_packet(req_id, &requests);

	let response = {
		let response = vec![Response::Signal(SignalResponse {
			signal: vec![1, 2, 3, 4],
		})];

		let limit = *flow_params.limit();
		let cost = flow_params.compute_cost_multi(requests.requests()).unwrap();

		let new_creds = limit - cost;

		let mut response_stream = RlpStream::new_list(3);
		response_stream.append(&req_id).append(&new_creds).append_list(&response);

		response_stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}

#[test]
fn proof_of_execution() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body.clone()));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &packet_body);
	}

	let req_id = 112;
	let mut request = Request::Execution(request::IncompleteExecutionRequest {
		block_hash: H256::default().into(),
		from: Address::default(),
		action: Action::Call(Address::default()),
		gas: 100.into(),
		gas_price: 0.into(),
		value: 0.into(),
		data: Vec::new(),
	});

	// first: a valid amount to request execution of.
	let requests = encode_single(request.clone());
	let request_body = make_packet(req_id, &requests);

	let response = {
		let limit = *flow_params.limit();
		let cost = flow_params.compute_cost_multi(requests.requests()).unwrap();

		let new_creds = limit - cost;

		let mut response_stream = RlpStream::new_list(3);
		response_stream.append(&req_id).append(&new_creds).begin_list(0);

		response_stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);

	// next: way too much requested gas.
	if let Request::Execution(ref mut req) = request {
		req.gas = 100_000_000.into();
	}
	let req_id = 113;
	let requests = encode_single(request.clone());
	let request_body = make_packet(req_id, &requests);

	let expected = Expect::Punish(1);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}

#[test]
fn id_guard() {
	use super::request_set::RequestSet;
	use super::ReqId;

	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let req_id_1 = ReqId(5143);
	let req_id_2 = ReqId(1111);

	let req = encode_single(Request::Headers(IncompleteHeadersRequest {
		start: HashOrNumber::Number(5u64).into(),
		max: 100,
		skip: 0,
		reverse: false,
	}));

	let peer_id = 9876;

	let mut pending_requests = RequestSet::default();

	pending_requests.insert(req_id_1, req.clone(), 0.into(), ::time::SteadyTime::now());
	pending_requests.insert(req_id_2, req, 1.into(), ::time::SteadyTime::now());

	proto.peers.write().insert(peer_id, ::parking_lot::Mutex::new(Peer {
		local_credits: flow_params.create_credits(),
		status: status(provider.client.chain_info()),
		capabilities: capabilities.clone(),
		remote_flow: Some((flow_params.create_credits(), (&*flow_params).clone())),
		sent_head: provider.client.chain_info().best_block_hash,
		last_update: ::time::SteadyTime::now(),
		pending_requests: pending_requests,
		failed_requests: Vec::new(),
		propagated_transactions: Default::default(),
		skip_update: false,
		local_flow: flow_params,
		awaiting_acknowledge: None,
	}));

	// first, malformed responses.
	{
		let mut stream = RlpStream::new_list(3);
		stream.append(&req_id_1.0);
		stream.append(&4_000_000usize);
		stream.begin_list(2).append(&125usize).append(&3usize);

		let packet = stream.out();
		assert!(proto.response(&peer_id, &Expect::Nothing, UntrustedRlp::new(&packet)).is_err());
	}

	// next, do an unexpected response.
	{
		let mut stream = RlpStream::new_list(3);
		stream.append(&10000usize);
		stream.append(&3_000_000usize);
		stream.begin_list(0);

		let packet = stream.out();
		assert!(proto.response(&peer_id, &Expect::Nothing, UntrustedRlp::new(&packet)).is_err());
	}

	// lastly, do a valid (but empty) response.
	{
		let mut stream = RlpStream::new_list(3);
		stream.append(&req_id_2.0);
		stream.append(&3_000_000usize);
		stream.begin_list(0);

		let packet = stream.out();
		assert!(proto.response(&peer_id, &Expect::Nothing, UntrustedRlp::new(&packet)).is_ok());
	}

	let peers = proto.peers.read();
	if let Some(ref peer_info) = peers.get(&peer_id) {
		let peer_info = peer_info.lock();
		assert!(peer_info.pending_requests.collect_ids::<Vec<_>>().is_empty());
		assert_eq!(peer_info.failed_requests, &[req_id_1]);
	}
}

#[test]
fn get_transaction_index() {
	let capabilities = capabilities();

	let (provider, proto) = setup(capabilities.clone());
	let flow_params = proto.flow_params.read().clone();

	let cur_status = status(provider.client.chain_info());

	{
		let packet_body = write_handshake(&cur_status, &capabilities, &proto);
		proto.on_connect(&1, &Expect::Send(1, packet::STATUS, packet_body.clone()));
		proto.handle_packet(&Expect::Nothing, &1, packet::STATUS, &packet_body);
	}

	let req_id = 112;
	let key1: H256 = U256::from(11223344).into();

	let request = Request::TransactionIndex(IncompleteTransactionIndexRequest {
		hash: key1.into(),
	});

	let requests = encode_single(request.clone());
	let request_body = make_packet(req_id, &requests);
	let response = {
		let response = vec![Response::TransactionIndex(TransactionIndexResponse {
			num: 100,
			hash: key1,
			index: 55,
		})];

		let new_creds = *flow_params.limit() - flow_params.compute_cost_multi(requests.requests()).unwrap();

		let mut response_stream = RlpStream::new_list(3);

		response_stream.append(&req_id).append(&new_creds).append_list(&response);
		response_stream.out()
	};

	let expected = Expect::Respond(packet::RESPONSE, response);
	proto.handle_packet(&expected, &1, packet::REQUEST, &request_body);
}
